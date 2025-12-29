import React, { useEffect, useState } from 'react';
export const UptimeDisplay = ({ service, period = 'month', className = '' }) => {
    const [uptimeData, setUptimeData] = useState([]);
    const [slaTargets, setSlaTargets] = useState([]);
    const [loading, setLoading] = useState(true);
    const [selectedPeriod, setSelectedPeriod] = useState(period);
    useEffect(() => {
        fetchUptimeData();
        fetchSLATargets();
    }, [service, selectedPeriod]);
    const fetchUptimeData = async () => {
        try {
            setLoading(true);
            const params = new URLSearchParams({
                period: selectedPeriod,
                ...(service && { service })
            });
            const response = await fetch(`/api/monitoring/uptime?${params}`);
            if (!response.ok)
                throw new Error('Failed to fetch uptime data');
            const data = await response.json();
            setUptimeData(data);
        }
        catch (error) {
            console.error('[UptimeDisplay] Failed to fetch uptime:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const fetchSLATargets = async () => {
        try {
            const params = service ? `?service=${service}` : '';
            const response = await fetch(`/api/monitoring/sla/targets${params}`);
            if (!response.ok)
                throw new Error('Failed to fetch SLA targets');
            const data = await response.json();
            setSlaTargets(data);
        }
        catch (error) {
            console.error('[UptimeDisplay] Failed to fetch SLA targets:', error);
        }
    };
    const getUptimeColor = (uptime) => {
        if (uptime >= 99.95)
            return '#10b981';
        if (uptime >= 99.9)
            return '#3b82f6';
        if (uptime >= 99.0)
            return '#f59e0b';
        return '#ef4444';
    };
    const getSLAStatusColor = (status) => {
        switch (status) {
            case 'met':
                return '#10b981';
            case 'at_risk':
                return '#f59e0b';
            case 'breached':
                return '#ef4444';
            default:
                return '#6b7280';
        }
    };
    const calculateOverallUptime = () => {
        if (uptimeData.length === 0)
            return 100;
        const total = uptimeData.reduce((sum, record) => sum + record.uptime, 0);
        return total / uptimeData.length;
    };
    const calculateDowntime = (uptime, totalTime) => {
        const downtimeMs = totalTime * (100 - uptime) / 100;
        const minutes = Math.floor(downtimeMs / 60000);
        const hours = Math.floor(minutes / 60);
        const days = Math.floor(hours / 24);
        if (days > 0)
            return `${days}d ${hours % 24}h`;
        if (hours > 0)
            return `${hours}h ${minutes % 60}m`;
        return `${minutes}m`;
    };
    const getPeriodDuration = (period) => {
        switch (period) {
            case 'hour':
                return 3600000;
            case 'day':
                return 86400000;
            case 'week':
                return 604800000;
            case 'month':
                return 2592000000;
            case 'year':
                return 31536000000;
            default:
                return 86400000;
        }
    };
    const overallUptime = calculateOverallUptime();
    if (loading) {
        return (React.createElement("div", { style: styles.loading },
            React.createElement("div", { style: styles.spinner }),
            React.createElement("p", null, "Loading uptime data...")));
    }
    return (React.createElement("div", { className: `uptime-display ${className}`, style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("h2", { style: styles.title }, "Uptime & SLA Monitoring"),
            React.createElement("div", { style: styles.periodSelector }, ['hour', 'day', 'week', 'month', 'year'].map((p) => (React.createElement("button", { key: p, style: {
                    ...styles.periodButton,
                    ...(selectedPeriod === p ? styles.periodButtonActive : {})
                }, onClick: () => setSelectedPeriod(p) }, p.charAt(0).toUpperCase() + p.slice(1)))))),
        React.createElement("div", { style: styles.overallCard },
            React.createElement("div", { style: styles.overallHeader },
                React.createElement("h3", { style: styles.overallTitle }, "Overall Uptime"),
                React.createElement("span", { style: styles.overallPeriod }, selectedPeriod)),
            React.createElement("div", { style: {
                    ...styles.overallValue,
                    color: getUptimeColor(overallUptime)
                } },
                overallUptime.toFixed(3),
                "%"),
            React.createElement("div", { style: styles.overallBar },
                React.createElement("div", { style: {
                        ...styles.overallBarFill,
                        width: `${overallUptime}%`,
                        backgroundColor: getUptimeColor(overallUptime)
                    } })),
            React.createElement("div", { style: styles.overallStats },
                React.createElement("div", { style: styles.overallStat },
                    React.createElement("span", { style: styles.overallStatLabel }, "Total Checks:"),
                    React.createElement("span", { style: styles.overallStatValue }, uptimeData.reduce((sum, r) => sum + r.totalChecks, 0).toLocaleString())),
                React.createElement("div", { style: styles.overallStat },
                    React.createElement("span", { style: styles.overallStatLabel }, "Failed:"),
                    React.createElement("span", { style: styles.overallStatValue }, uptimeData.reduce((sum, r) => sum + r.failedChecks, 0).toLocaleString())),
                React.createElement("div", { style: styles.overallStat },
                    React.createElement("span", { style: styles.overallStatLabel }, "Downtime:"),
                    React.createElement("span", { style: styles.overallStatValue }, calculateDowntime(overallUptime, getPeriodDuration(selectedPeriod)))))),
        React.createElement("div", { style: styles.section },
            React.createElement("h3", { style: styles.sectionTitle }, "Service Uptime"),
            React.createElement("div", { style: styles.uptimeGrid }, uptimeData.map((record, index) => (React.createElement("div", { key: index, style: styles.uptimeCard },
                React.createElement("div", { style: styles.uptimeHeader },
                    React.createElement("span", { style: styles.uptimeService }, record.service),
                    React.createElement("span", { style: {
                            ...styles.uptimeValue,
                            color: getUptimeColor(record.uptime)
                        } },
                        record.uptime.toFixed(3),
                        "%")),
                React.createElement("div", { style: styles.uptimeBar },
                    React.createElement("div", { style: {
                            ...styles.uptimeBarFill,
                            width: `${record.uptime}%`,
                            backgroundColor: getUptimeColor(record.uptime)
                        } })),
                React.createElement("div", { style: styles.uptimeStats },
                    React.createElement("div", { style: styles.uptimeStat },
                        React.createElement("span", null, "Checks:"),
                        React.createElement("span", null, record.totalChecks)),
                    React.createElement("div", { style: styles.uptimeStat },
                        React.createElement("span", null, "Failed:"),
                        React.createElement("span", null, record.failedChecks)),
                    React.createElement("div", { style: styles.uptimeStat },
                        React.createElement("span", null, "Avg Response:"),
                        React.createElement("span", null,
                            record.averageResponseTime.toFixed(0),
                            "ms")))))))),
        slaTargets.length > 0 && (React.createElement("div", { style: styles.section },
            React.createElement("h3", { style: styles.sectionTitle }, "SLA Targets"),
            React.createElement("div", { style: styles.slaGrid }, slaTargets.map((target) => (React.createElement("div", { key: target.id, style: styles.slaCard },
                React.createElement("div", { style: styles.slaHeader },
                    React.createElement("div", null,
                        React.createElement("div", { style: styles.slaService }, target.service),
                        React.createElement("div", { style: styles.slaMetric }, target.metric)),
                    React.createElement("div", { style: {
                            ...styles.slaStatus,
                            backgroundColor: `${getSLAStatusColor(target.status)}20`,
                            color: getSLAStatusColor(target.status)
                        } }, target.status.replace('_', ' ').toUpperCase())),
                React.createElement("div", { style: styles.slaProgress },
                    React.createElement("div", { style: styles.slaProgressHeader },
                        React.createElement("span", { style: styles.slaProgressLabel }, "Current"),
                        React.createElement("span", { style: styles.slaProgressValue },
                            target.current.toFixed(2),
                            "%")),
                    React.createElement("div", { style: styles.slaProgressBar },
                        React.createElement("div", { style: {
                                ...styles.slaProgressFill,
                                width: `${Math.min((target.current / target.target) * 100, 100)}%`,
                                backgroundColor: getSLAStatusColor(target.status)
                            } }),
                        React.createElement("div", { style: {
                                ...styles.slaTarget,
                                left: `${Math.min((target.target / 100) * 100, 100)}%`
                            } })),
                    React.createElement("div", { style: styles.slaProgressFooter },
                        React.createElement("span", { style: styles.slaTargetLabel },
                            "Target: ",
                            target.target,
                            "%"))),
                React.createElement("div", { style: styles.slaBudget },
                    React.createElement("div", { style: styles.slaBudgetItem },
                        React.createElement("span", { style: styles.slaBudgetLabel }, "Error Budget:"),
                        React.createElement("span", { style: styles.slaBudgetValue },
                            target.errorBudget.toFixed(2),
                            "%")),
                    React.createElement("div", { style: styles.slaBudgetItem },
                        React.createElement("span", { style: styles.slaBudgetLabel }, "Remaining:"),
                        React.createElement("span", { style: {
                                ...styles.slaBudgetValue,
                                color: target.errorBudgetRemaining > 0 ? '#10b981' : '#ef4444'
                            } },
                            target.errorBudgetRemaining.toFixed(2),
                            "%"))))))))),
        React.createElement("div", { style: styles.section },
            React.createElement("h3", { style: styles.sectionTitle }, "Uptime History"),
            React.createElement("div", { style: styles.calendar }, uptimeData.map((record, index) => {
                const startDate = new Date(record.startTime);
                const endDate = new Date(record.endTime);
                return (React.createElement("div", { key: index, style: styles.calendarDay, title: `${record.service}: ${record.uptime.toFixed(3)}%` },
                    React.createElement("div", { style: {
                            ...styles.calendarDayBar,
                            backgroundColor: getUptimeColor(record.uptime),
                            opacity: 0.2 + (record.uptime / 100) * 0.8
                        } }),
                    React.createElement("div", { style: styles.calendarDayLabel }, startDate.toLocaleDateString(undefined, { month: 'short', day: 'numeric' }))));
            })))));
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
        marginBottom: '24px',
        flexWrap: 'wrap',
        gap: '16px'
    },
    title: {
        fontSize: '24px',
        fontWeight: 700,
        color: '#111827',
        margin: 0
    },
    periodSelector: {
        display: 'flex',
        gap: '4px',
        backgroundColor: '#f3f4f6',
        borderRadius: '8px',
        padding: '4px'
    },
    periodButton: {
        padding: '6px 16px',
        border: 'none',
        background: 'transparent',
        borderRadius: '6px',
        fontSize: '13px',
        fontWeight: 500,
        color: '#6b7280',
        cursor: 'pointer',
        transition: 'all 0.2s'
    },
    periodButtonActive: {
        backgroundColor: '#fff',
        color: '#111827'
    },
    overallCard: {
        backgroundColor: '#fff',
        border: '2px solid #e5e7eb',
        borderRadius: '12px',
        padding: '32px',
        marginBottom: '32px'
    },
    overallHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '16px'
    },
    overallTitle: {
        fontSize: '18px',
        fontWeight: 600,
        color: '#111827',
        margin: 0
    },
    overallPeriod: {
        fontSize: '14px',
        color: '#6b7280',
        textTransform: 'capitalize'
    },
    overallValue: {
        fontSize: '48px',
        fontWeight: 700,
        marginBottom: '16px'
    },
    overallBar: {
        height: '12px',
        backgroundColor: '#f3f4f6',
        borderRadius: '6px',
        overflow: 'hidden',
        marginBottom: '16px'
    },
    overallBarFill: {
        height: '100%',
        transition: 'width 0.3s ease'
    },
    overallStats: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
        gap: '16px'
    },
    overallStat: {
        display: 'flex',
        flexDirection: 'column',
        gap: '4px'
    },
    overallStatLabel: {
        fontSize: '13px',
        color: '#6b7280'
    },
    overallStatValue: {
        fontSize: '18px',
        fontWeight: 600,
        color: '#111827'
    },
    section: {
        marginBottom: '32px'
    },
    sectionTitle: {
        fontSize: '18px',
        fontWeight: 600,
        color: '#111827',
        marginBottom: '16px'
    },
    uptimeGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
        gap: '16px'
    },
    uptimeCard: {
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px',
        padding: '20px'
    },
    uptimeHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '12px'
    },
    uptimeService: {
        fontSize: '15px',
        fontWeight: 600,
        color: '#111827'
    },
    uptimeValue: {
        fontSize: '20px',
        fontWeight: 700
    },
    uptimeBar: {
        height: '8px',
        backgroundColor: '#f3f4f6',
        borderRadius: '4px',
        overflow: 'hidden',
        marginBottom: '12px'
    },
    uptimeBarFill: {
        height: '100%',
        transition: 'width 0.3s ease'
    },
    uptimeStats: {
        display: 'grid',
        gridTemplateColumns: 'repeat(3, 1fr)',
        gap: '8px',
        fontSize: '12px',
        color: '#6b7280'
    },
    uptimeStat: {
        display: 'flex',
        flexDirection: 'column',
        gap: '2px'
    },
    slaGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(350px, 1fr))',
        gap: '16px'
    },
    slaCard: {
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px',
        padding: '20px'
    },
    slaHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'flex-start',
        marginBottom: '16px'
    },
    slaService: {
        fontSize: '16px',
        fontWeight: 600,
        color: '#111827',
        marginBottom: '4px'
    },
    slaMetric: {
        fontSize: '13px',
        color: '#6b7280'
    },
    slaStatus: {
        fontSize: '11px',
        fontWeight: 600,
        padding: '4px 10px',
        borderRadius: '12px'
    },
    slaProgress: {
        marginBottom: '16px'
    },
    slaProgressHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        marginBottom: '8px'
    },
    slaProgressLabel: {
        fontSize: '13px',
        color: '#6b7280'
    },
    slaProgressValue: {
        fontSize: '16px',
        fontWeight: 600,
        color: '#111827'
    },
    slaProgressBar: {
        position: 'relative',
        height: '10px',
        backgroundColor: '#f3f4f6',
        borderRadius: '5px',
        overflow: 'visible',
        marginBottom: '8px'
    },
    slaProgressFill: {
        height: '100%',
        borderRadius: '5px',
        transition: 'width 0.3s ease'
    },
    slaTarget: {
        position: 'absolute',
        top: '-2px',
        width: '2px',
        height: '14px',
        backgroundColor: '#111827',
        transform: 'translateX(-1px)'
    },
    slaProgressFooter: {
        display: 'flex',
        justifyContent: 'flex-end'
    },
    slaTargetLabel: {
        fontSize: '12px',
        color: '#6b7280'
    },
    slaBudget: {
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        gap: '12px',
        paddingTop: '16px',
        borderTop: '1px solid #e5e7eb'
    },
    slaBudgetItem: {
        display: 'flex',
        flexDirection: 'column',
        gap: '4px'
    },
    slaBudgetLabel: {
        fontSize: '12px',
        color: '#6b7280'
    },
    slaBudgetValue: {
        fontSize: '16px',
        fontWeight: 600,
        color: '#111827'
    },
    calendar: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(60px, 1fr))',
        gap: '8px'
    },
    calendarDay: {
        position: 'relative',
        height: '60px',
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '6px',
        overflow: 'hidden',
        cursor: 'pointer',
        transition: 'transform 0.2s'
    },
    calendarDayBar: {
        position: 'absolute',
        bottom: 0,
        left: 0,
        right: 0,
        height: '100%'
    },
    calendarDayLabel: {
        position: 'relative',
        fontSize: '11px',
        color: '#374151',
        fontWeight: 500,
        padding: '4px',
        textAlign: 'center'
    }
};
export default UptimeDisplay;
//# sourceMappingURL=UptimeDisplay.js.map