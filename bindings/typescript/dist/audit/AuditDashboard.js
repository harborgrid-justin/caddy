import React, { useState, useEffect, useMemo } from 'react';
export const AuditDashboard = ({ organizationId, onNavigate, }) => {
    const [metrics, setMetrics] = useState(null);
    const [recentEvents, setRecentEvents] = useState([]);
    const [loading, setLoading] = useState(true);
    const [timeRange, setTimeRange] = useState({
        start: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000),
        end: new Date(),
    });
    useEffect(() => {
        loadDashboardData();
    }, [organizationId, timeRange]);
    const loadDashboardData = async () => {
        setLoading(true);
        try {
            const params = new URLSearchParams({
                start_date: timeRange.start.toISOString(),
                end_date: timeRange.end.toISOString(),
                ...(organizationId && { organization_id: organizationId }),
            });
            const [metricsRes, eventsRes] = await Promise.all([
                fetch(`/api/audit/metrics?${params}`),
                fetch(`/api/audit/events?${params}&limit=10`),
            ]);
            const metricsData = await metricsRes.json();
            const eventsData = await eventsRes.json();
            setMetrics(metricsData);
            setRecentEvents(eventsData.events || []);
        }
        catch (error) {
            console.error('Failed to load dashboard data:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const severityDistribution = useMemo(() => {
        if (!metrics)
            return [];
        return Object.entries(metrics.events_by_severity).map(([severity, count]) => ({
            severity: severity,
            count,
            percentage: (count / metrics.total_events) * 100,
        }));
    }, [metrics]);
    const topEventTypes = useMemo(() => {
        if (!metrics)
            return [];
        return Object.entries(metrics.events_by_type)
            .sort(([, a], [, b]) => b - a)
            .slice(0, 5)
            .map(([type, count]) => ({
            type: type,
            count,
        }));
    }, [metrics]);
    if (loading) {
        return (React.createElement("div", { className: "audit-dashboard loading" },
            React.createElement("div", { className: "loading-spinner" }),
            React.createElement("p", null, "Loading audit dashboard...")));
    }
    return (React.createElement("div", { className: "audit-dashboard" },
        React.createElement("header", { className: "dashboard-header" },
            React.createElement("div", null,
                React.createElement("h1", null, "Audit Dashboard"),
                React.createElement("p", { className: "subtitle" }, "Comprehensive audit trail and security monitoring")),
            React.createElement("div", { className: "dashboard-controls" },
                React.createElement(TimeRangeSelector, { value: timeRange, onChange: setTimeRange }),
                React.createElement("button", { className: "refresh-button", onClick: loadDashboardData }, "Refresh"))),
        metrics && (React.createElement("div", { className: "metrics-grid" },
            React.createElement(MetricCard, { title: "Total Events", value: formatNumber(metrics.total_events), icon: "\uD83D\uDCCA", trend: "neutral" }),
            React.createElement(MetricCard, { title: "Anomalies Detected", value: metrics.anomalies_detected.toString(), icon: "\u26A0\uFE0F", trend: metrics.anomalies_detected > 0 ? 'danger' : 'success', onClick: () => onNavigate?.('anomalies') }),
            React.createElement(MetricCard, { title: "Unique Users", value: metrics.unique_users.toString(), icon: "\uD83D\uDC65", trend: "neutral" }),
            React.createElement(MetricCard, { title: "High Risk Events", value: metrics.high_risk_events.toString(), icon: "\uD83D\uDD25", trend: metrics.high_risk_events > 0 ? 'warning' : 'success', onClick: () => onNavigate?.('high-risk') }),
            React.createElement(MetricCard, { title: "Failed Events", value: metrics.failed_events.toString(), icon: "\u274C", trend: metrics.failed_events > 0 ? 'warning' : 'success' }),
            React.createElement(MetricCard, { title: "Unique Resources", value: metrics.unique_resources.toString(), icon: "\uD83D\uDCE6", trend: "neutral" }))),
        React.createElement("div", { className: "charts-section" },
            React.createElement("div", { className: "chart-container" },
                React.createElement("h3", null, "Event Timeline"),
                metrics && (React.createElement(EventTimelineChart, { data: metrics.timeline }))),
            React.createElement("div", { className: "chart-container" },
                React.createElement("h3", null, "Events by Severity"),
                React.createElement(SeverityDistributionChart, { data: severityDistribution }))),
        React.createElement("div", { className: "charts-section" },
            React.createElement("div", { className: "chart-container" },
                React.createElement("h3", null, "Top Event Types"),
                React.createElement(TopEventsChart, { data: topEventTypes })),
            React.createElement("div", { className: "chart-container" },
                React.createElement("h3", null, "Most Active Users"),
                metrics && React.createElement(TopUsersTable, { users: metrics.top_users }))),
        React.createElement("div", { className: "recent-events-section" },
            React.createElement("div", { className: "section-header" },
                React.createElement("h3", null, "Recent Audit Events"),
                React.createElement("button", { onClick: () => onNavigate?.('logs') }, "View All Logs")),
            React.createElement(RecentEventsTable, { events: recentEvents })),
        React.createElement("div", { className: "quick-actions" },
            React.createElement(QuickActionButton, { icon: "\uD83D\uDD0D", title: "Search Logs", description: "Advanced audit log search", onClick: () => onNavigate?.('logs') }),
            React.createElement(QuickActionButton, { icon: "\uD83D\uDCCB", title: "Compliance Reports", description: "Generate compliance reports", onClick: () => onNavigate?.('compliance') }),
            React.createElement(QuickActionButton, { icon: "\uD83D\uDCE4", title: "Export Logs", description: "Export audit data", onClick: () => onNavigate?.('export') }),
            React.createElement(QuickActionButton, { icon: "\uD83D\uDD14", title: "Configure Alerts", description: "Set up audit alerts", onClick: () => onNavigate?.('alerts') }))));
};
function TimeRangeSelector({ value, onChange, }) {
    const presets = [
        { label: 'Last 24 Hours', hours: 24 },
        { label: 'Last 7 Days', days: 7 },
        { label: 'Last 30 Days', days: 30 },
        { label: 'Last 90 Days', days: 90 },
    ];
    return (React.createElement("div", { className: "time-range-selector" },
        React.createElement("select", { onChange: (e) => {
                const preset = presets[parseInt(e.target.value)];
                if (preset) {
                    const ms = (preset.hours || 0) * 60 * 60 * 1000 + (preset.days || 0) * 24 * 60 * 60 * 1000;
                    onChange({
                        start: new Date(Date.now() - ms),
                        end: new Date(),
                    });
                }
            } }, presets.map((preset, index) => (React.createElement("option", { key: index, value: index }, preset.label))))));
}
function MetricCard({ title, value, icon, trend, onClick, }) {
    return (React.createElement("div", { className: `metric-card ${trend ? `trend-${trend}` : ''} ${onClick ? 'clickable' : ''}`, onClick: onClick },
        icon && React.createElement("div", { className: "metric-icon" }, icon),
        React.createElement("div", { className: "metric-content" },
            React.createElement("h4", null, title),
            React.createElement("div", { className: "metric-value" }, value))));
}
function EventTimelineChart({ data, }) {
    const maxCount = Math.max(...data.map((d) => d.count), 1);
    return (React.createElement("div", { className: "timeline-chart" }, data.map((point, index) => (React.createElement("div", { key: index, className: "timeline-bar" },
        React.createElement("div", { className: "bar", style: { height: `${(point.count / maxCount) * 100}%` }, title: `${point.count} events` }),
        point.anomalies > 0 && (React.createElement("div", { className: "anomaly-indicator", title: `${point.anomalies} anomalies` })),
        React.createElement("div", { className: "bar-label" }, new Date(point.timestamp).toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
        })))))));
}
function SeverityDistributionChart({ data, }) {
    const severityColors = {
        low: '#10b981',
        medium: '#f59e0b',
        high: '#f97316',
        critical: '#ef4444',
    };
    return (React.createElement("div", { className: "severity-chart" },
        React.createElement("div", { className: "severity-bars" }, data.map((item) => (React.createElement("div", { key: item.severity, className: "severity-item" },
            React.createElement("div", { className: "severity-label" },
                React.createElement("span", { className: "severity-name" }, item.severity),
                React.createElement("span", { className: "severity-count" }, item.count)),
            React.createElement("div", { className: "severity-bar-container" },
                React.createElement("div", { className: "severity-bar", style: {
                        width: `${item.percentage}%`,
                        backgroundColor: severityColors[item.severity],
                    } })),
            React.createElement("span", { className: "severity-percentage" },
                item.percentage.toFixed(1),
                "%")))))));
}
function TopEventsChart({ data, }) {
    const maxCount = Math.max(...data.map((d) => d.count), 1);
    return (React.createElement("div", { className: "top-events-chart" }, data.map((item) => (React.createElement("div", { key: item.type, className: "event-item" },
        React.createElement("div", { className: "event-label" }, formatEventType(item.type)),
        React.createElement("div", { className: "event-bar-container" },
            React.createElement("div", { className: "event-bar", style: { width: `${(item.count / maxCount) * 100}%` } })),
        React.createElement("div", { className: "event-count" }, item.count))))));
}
function TopUsersTable({ users, }) {
    return (React.createElement("table", { className: "top-users-table" },
        React.createElement("thead", null,
            React.createElement("tr", null,
                React.createElement("th", null, "User"),
                React.createElement("th", null, "Events"),
                React.createElement("th", null, "Risk Score"))),
        React.createElement("tbody", null, users.map((user) => (React.createElement("tr", { key: user.user_id },
            React.createElement("td", null, user.user_email),
            React.createElement("td", null, user.event_count),
            React.createElement("td", null,
                React.createElement(RiskScoreBadge, { score: user.risk_score }))))))));
}
function RecentEventsTable({ events }) {
    return (React.createElement("table", { className: "recent-events-table" },
        React.createElement("thead", null,
            React.createElement("tr", null,
                React.createElement("th", null, "Time"),
                React.createElement("th", null, "Event Type"),
                React.createElement("th", null, "User"),
                React.createElement("th", null, "Resource"),
                React.createElement("th", null, "Status"),
                React.createElement("th", null, "Severity"))),
        React.createElement("tbody", null, events.map((event) => (React.createElement("tr", { key: event.id },
            React.createElement("td", null, formatTimestamp(event.timestamp)),
            React.createElement("td", null, formatEventType(event.event_type)),
            React.createElement("td", null, event.user_email || 'System'),
            React.createElement("td", null, event.resource_name || event.resource_id || '-'),
            React.createElement("td", null,
                React.createElement(StatusBadge, { status: event.status })),
            React.createElement("td", null,
                React.createElement(SeverityBadge, { severity: event.severity }))))))));
}
function QuickActionButton({ icon, title, description, onClick, }) {
    return (React.createElement("button", { className: "quick-action-button", onClick: onClick },
        React.createElement("div", { className: "action-icon" }, icon),
        React.createElement("div", { className: "action-content" },
            React.createElement("div", { className: "action-title" }, title),
            React.createElement("div", { className: "action-description" }, description))));
}
function StatusBadge({ status }) {
    const colors = {
        success: 'green',
        failure: 'red',
        pending: 'yellow',
        blocked: 'gray',
    };
    return (React.createElement("span", { className: `badge badge-${colors[status] || 'gray'}` }, status));
}
function SeverityBadge({ severity }) {
    const colors = {
        low: 'green',
        medium: 'yellow',
        high: 'orange',
        critical: 'red',
    };
    return (React.createElement("span", { className: `badge badge-${colors[severity]}` }, severity));
}
function RiskScoreBadge({ score }) {
    const getColor = (score) => {
        if (score >= 80)
            return 'red';
        if (score >= 60)
            return 'orange';
        if (score >= 40)
            return 'yellow';
        return 'green';
    };
    return (React.createElement("span", { className: `badge badge-${getColor(score)}` }, score.toFixed(0)));
}
function formatNumber(num) {
    return new Intl.NumberFormat('en-US').format(num);
}
function formatTimestamp(timestamp) {
    const date = new Date(timestamp);
    return new Intl.DateTimeFormat('en-US', {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
    }).format(date);
}
function formatEventType(eventType) {
    return eventType
        .split('.')
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join(' ');
}
//# sourceMappingURL=AuditDashboard.js.map