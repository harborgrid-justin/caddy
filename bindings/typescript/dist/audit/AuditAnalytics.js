import React, { useState, useEffect, useMemo } from 'react';
export const AuditAnalytics = ({ organizationId }) => {
    const [analytics, setAnalytics] = useState(null);
    const [loading, setLoading] = useState(true);
    const [timeRange, setTimeRange] = useState({
        start: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000),
        end: new Date(),
    });
    const [selectedTab, setSelectedTab] = useState('trends');
    useEffect(() => {
        loadAnalytics();
    }, [timeRange, organizationId]);
    const loadAnalytics = async () => {
        setLoading(true);
        try {
            const params = new URLSearchParams({
                start_date: timeRange.start.toISOString(),
                end_date: timeRange.end.toISOString(),
                ...(organizationId && { organization_id: organizationId }),
            });
            const response = await fetch(`/api/audit/analytics?${params}`);
            const data = await response.json();
            setAnalytics(data);
        }
        catch (error) {
            console.error('Failed to load analytics:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const totalEvents = useMemo(() => {
        if (!analytics)
            return 0;
        return analytics.event_trends.reduce((sum, trend) => sum + trend.total, 0);
    }, [analytics]);
    const avgEventsPerDay = useMemo(() => {
        if (!analytics || analytics.event_trends.length === 0)
            return 0;
        return totalEvents / analytics.event_trends.length;
    }, [analytics, totalEvents]);
    if (loading) {
        return (React.createElement("div", { className: "audit-analytics loading" },
            React.createElement("div", { className: "loading-spinner" }),
            React.createElement("p", null, "Loading analytics...")));
    }
    if (!analytics) {
        return (React.createElement("div", { className: "audit-analytics error" },
            React.createElement("h2", null, "Failed to Load Analytics"),
            React.createElement("p", null, "Unable to load analytics data. Please try again.")));
    }
    return (React.createElement("div", { className: "audit-analytics" },
        React.createElement("div", { className: "analytics-header" },
            React.createElement("div", null,
                React.createElement("h2", null, "Audit Analytics"),
                React.createElement("p", { className: "subtitle" }, "Advanced analytics and trend analysis for audit events")),
            React.createElement(TimeRangeSelector, { value: timeRange, onChange: setTimeRange })),
        React.createElement("div", { className: "summary-stats" },
            React.createElement(StatCard, { title: "Total Events", value: formatNumber(totalEvents), icon: "\uD83D\uDCCA", trend: "neutral" }),
            React.createElement(StatCard, { title: "Avg Events/Day", value: formatNumber(Math.round(avgEventsPerDay)), icon: "\uD83D\uDCC8", trend: "neutral" }),
            React.createElement(StatCard, { title: "Total Anomalies", value: analytics.security_insights.total_anomalies.toString(), icon: "\u26A0\uFE0F", trend: analytics.security_insights.total_anomalies > 0 ? 'danger' : 'success' }),
            React.createElement(StatCard, { title: "High Risk Events", value: analytics.security_insights.high_risk_events.toString(), icon: "\uD83D\uDD25", trend: analytics.security_insights.high_risk_events > 0 ? 'warning' : 'success' })),
        React.createElement("div", { className: "analytics-tabs" },
            React.createElement("button", { className: `tab ${selectedTab === 'trends' ? 'active' : ''}`, onClick: () => setSelectedTab('trends') }, "Event Trends"),
            React.createElement("button", { className: `tab ${selectedTab === 'users' ? 'active' : ''}`, onClick: () => setSelectedTab('users') }, "User Behavior"),
            React.createElement("button", { className: `tab ${selectedTab === 'resources' ? 'active' : ''}`, onClick: () => setSelectedTab('resources') }, "Resource Usage"),
            React.createElement("button", { className: `tab ${selectedTab === 'security' ? 'active' : ''}`, onClick: () => setSelectedTab('security') }, "Security Insights")),
        React.createElement("div", { className: "analytics-content" },
            selectedTab === 'trends' && React.createElement(EventTrendsTab, { analytics: analytics }),
            selectedTab === 'users' && React.createElement(UserBehaviorTab, { analytics: analytics }),
            selectedTab === 'resources' && React.createElement(ResourceUsageTab, { analytics: analytics }),
            selectedTab === 'security' && React.createElement(SecurityInsightsTab, { analytics: analytics }))));
};
function EventTrendsTab({ analytics }) {
    const timelineData = analytics.event_trends.map(trend => ({
        date: trend.date,
        total: trend.total,
        anomalies: trend.by_severity.high + trend.by_severity.critical,
        high_risk: trend.by_severity.critical,
    }));
    return (React.createElement("div", { className: "trends-tab" },
        React.createElement("div", { className: "chart-section" },
            React.createElement("h3", null, "Event Timeline"),
            React.createElement(EventTimelineChart, { data: timelineData })),
        React.createElement("div", { className: "charts-row" },
            React.createElement("div", { className: "chart-section" },
                React.createElement("h3", null, "Events by Severity"),
                React.createElement(SeverityTrendsChart, { data: analytics.event_trends })),
            React.createElement("div", { className: "chart-section" },
                React.createElement("h3", null, "Event Status Distribution"),
                React.createElement(StatusTrendsChart, { data: analytics.event_trends }))),
        React.createElement("div", { className: "trends-summary" },
            React.createElement("h3", null, "Key Trends"),
            React.createElement("div", { className: "trends-grid" },
                React.createElement(TrendCard, { title: "Peak Activity", value: findPeakActivity(analytics.event_trends), icon: "\uD83D\uDCC8" }),
                React.createElement(TrendCard, { title: "Anomaly Rate", value: calculateAnomalyRate(timelineData), icon: "\u26A0\uFE0F" }),
                React.createElement(TrendCard, { title: "High Risk Rate", value: calculateHighRiskRate(timelineData), icon: "\uD83D\uDD25" })))));
}
function UserBehaviorTab({ analytics }) {
    const [sortBy, setSortBy] = useState('events');
    const sortedUsers = useMemo(() => {
        const users = [...analytics.user_analytics];
        switch (sortBy) {
            case 'risk':
                return users.sort((a, b) => b.risk_score - a.risk_score);
            case 'anomalies':
                return users.sort((a, b) => b.anomaly_count - a.anomaly_count);
            default:
                return users.sort((a, b) => b.total_events - a.total_events);
        }
    }, [analytics.user_analytics, sortBy]);
    return (React.createElement("div", { className: "user-behavior-tab" },
        React.createElement("div", { className: "tab-controls" },
            React.createElement("label", null, "Sort by:"),
            React.createElement("select", { value: sortBy, onChange: (e) => setSortBy(e.target.value) },
                React.createElement("option", { value: "events" }, "Total Events"),
                React.createElement("option", { value: "risk" }, "Risk Score"),
                React.createElement("option", { value: "anomalies" }, "Anomalies"))),
        React.createElement("table", { className: "user-analytics-table" },
            React.createElement("thead", null,
                React.createElement("tr", null,
                    React.createElement("th", null, "User"),
                    React.createElement("th", null, "Total Events"),
                    React.createElement("th", null, "Logins"),
                    React.createElement("th", null, "Failed Logins"),
                    React.createElement("th", null, "Data Accessed"),
                    React.createElement("th", null, "Data Modified"),
                    React.createElement("th", null, "Anomalies"),
                    React.createElement("th", null, "Risk Score"),
                    React.createElement("th", null, "Last Activity"))),
            React.createElement("tbody", null, sortedUsers.map((user) => (React.createElement("tr", { key: user.user_id },
                React.createElement("td", null,
                    React.createElement("div", { className: "user-info" },
                        React.createElement("div", { className: "user-email" }, user.user_email),
                        React.createElement("div", { className: "user-id" }, user.user_id))),
                React.createElement("td", null, formatNumber(user.total_events)),
                React.createElement("td", null, user.login_count),
                React.createElement("td", null,
                    React.createElement("span", { className: `failed-logins ${user.failed_login_count > 3 ? 'high' : ''}` }, user.failed_login_count)),
                React.createElement("td", null, formatNumber(user.data_accessed_count)),
                React.createElement("td", null, formatNumber(user.data_modified_count)),
                React.createElement("td", null,
                    React.createElement("span", { className: `anomaly-count ${user.anomaly_count > 0 ? 'has-anomalies' : ''}` }, user.anomaly_count)),
                React.createElement("td", null,
                    React.createElement(RiskScoreBadge, { score: user.risk_score })),
                React.createElement("td", null, formatTimestamp(user.last_activity)))))))));
}
function ResourceUsageTab({ analytics }) {
    return (React.createElement("div", { className: "resource-usage-tab" },
        React.createElement("table", { className: "resource-analytics-table" },
            React.createElement("thead", null,
                React.createElement("tr", null,
                    React.createElement("th", null, "Resource Type"),
                    React.createElement("th", null, "Resource ID"),
                    React.createElement("th", null, "Total Accesses"),
                    React.createElement("th", null, "Unique Users"),
                    React.createElement("th", null, "Modifications"),
                    React.createElement("th", null, "Last Accessed"))),
            React.createElement("tbody", null, analytics.resource_analytics.map((resource, index) => (React.createElement("tr", { key: index },
                React.createElement("td", null,
                    React.createElement("span", { className: "resource-type-badge" }, resource.resource_type)),
                React.createElement("td", { className: "resource-id" }, resource.resource_id),
                React.createElement("td", null, formatNumber(resource.total_accesses)),
                React.createElement("td", null, resource.unique_users),
                React.createElement("td", null, resource.modifications),
                React.createElement("td", null, formatTimestamp(resource.last_accessed))))))),
        React.createElement("div", { className: "resource-summary" },
            React.createElement("h3", null, "Resource Summary"),
            React.createElement("div", { className: "summary-grid" },
                React.createElement("div", { className: "summary-card" },
                    React.createElement("div", { className: "summary-value" }, analytics.resource_analytics.length),
                    React.createElement("div", { className: "summary-label" }, "Total Resources")),
                React.createElement("div", { className: "summary-card" },
                    React.createElement("div", { className: "summary-value" }, analytics.resource_analytics.reduce((sum, r) => sum + r.total_accesses, 0)),
                    React.createElement("div", { className: "summary-label" }, "Total Accesses")),
                React.createElement("div", { className: "summary-card" },
                    React.createElement("div", { className: "summary-value" }, analytics.resource_analytics.reduce((sum, r) => sum + r.modifications, 0)),
                    React.createElement("div", { className: "summary-label" }, "Total Modifications"))))));
}
function SecurityInsightsTab({ analytics }) {
    return (React.createElement("div", { className: "security-insights-tab" },
        React.createElement("div", { className: "security-metrics" },
            React.createElement(MetricCard, { title: "Total Anomalies", value: analytics.security_insights.total_anomalies, color: "orange" }),
            React.createElement(MetricCard, { title: "Breach Attempts", value: analytics.security_insights.breach_attempts, color: "red" }),
            React.createElement(MetricCard, { title: "Unauthorized Access", value: analytics.security_insights.unauthorized_access_attempts, color: "red" }),
            React.createElement(MetricCard, { title: "Suspicious Activities", value: analytics.security_insights.suspicious_activities, color: "yellow" })),
        React.createElement("div", { className: "security-patterns" },
            React.createElement("h3", null, "Security Patterns"),
            analytics.security_insights.patterns.length === 0 ? (React.createElement("div", { className: "empty-state" },
                React.createElement("p", null, "No security patterns detected"))) : (analytics.security_insights.patterns.map((pattern, index) => (React.createElement("div", { key: index, className: `pattern-card risk-${pattern.risk_level}` },
                React.createElement("div", { className: "pattern-header" },
                    React.createElement("h4", null, pattern.type),
                    React.createElement("span", { className: `risk-badge risk-${pattern.risk_level}` }, pattern.risk_level.toUpperCase())),
                React.createElement("p", { className: "pattern-description" }, pattern.description),
                React.createElement("div", { className: "pattern-stats" },
                    React.createElement("span", null,
                        React.createElement("strong", null, "Occurrences:"),
                        " ",
                        pattern.occurrence_count),
                    React.createElement("span", null,
                        React.createElement("strong", null, "Affected Users:"),
                        " ",
                        pattern.affected_users.length)),
                React.createElement("div", { className: "pattern-recommendations" },
                    React.createElement("strong", null, "Recommendations:"),
                    React.createElement("ul", null, pattern.recommendations.map((rec, idx) => (React.createElement("li", { key: idx }, rec)))))))))),
        React.createElement("div", { className: "compliance-insights" },
            React.createElement("h3", null, "Compliance Overview"),
            React.createElement("div", { className: "compliance-grid" }, analytics.compliance_insights.map((insight) => (React.createElement("div", { key: insight.framework, className: "compliance-card" },
                React.createElement("div", { className: "compliance-header" },
                    React.createElement("h4", null, insight.framework),
                    React.createElement("div", { className: "compliance-percentage" },
                        insight.compliant_percentage.toFixed(1),
                        "%")),
                React.createElement("div", { className: "compliance-stats" },
                    React.createElement("div", { className: "stat" },
                        React.createElement("label", null, "Violations:"),
                        React.createElement("span", { className: insight.violations > 0 ? 'has-violations' : '' }, insight.violations)),
                    insight.at_risk_requirements.length > 0 && (React.createElement("div", { className: "at-risk" },
                        React.createElement("label", null, "At Risk:"),
                        React.createElement("span", null,
                            insight.at_risk_requirements.length,
                            " requirements")))))))))));
}
function EventTimelineChart({ data, }) {
    const maxCount = Math.max(...data.map((d) => d.total), 1);
    return (React.createElement("div", { className: "timeline-chart" }, data.map((point, index) => (React.createElement("div", { key: index, className: "timeline-bar-group" },
        React.createElement("div", { className: "bar-stack" },
            React.createElement("div", { className: "bar total-bar", style: { height: `${(point.total / maxCount) * 100}%` }, title: `${point.total} events` }),
            point.anomalies > 0 && (React.createElement("div", { className: "anomaly-overlay", title: `${point.anomalies} anomalies` }))),
        React.createElement("div", { className: "bar-label" }, new Date(point.date).toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
        })))))));
}
function SeverityTrendsChart({ data, }) {
    const severityColors = {
        low: '#10b981',
        medium: '#f59e0b',
        high: '#f97316',
        critical: '#ef4444',
    };
    return (React.createElement("div", { className: "severity-trends-chart" }, data.map((point, index) => (React.createElement("div", { key: index, className: "severity-bar-group" },
        React.createElement("div", { className: "stacked-bar" }, Object.entries(severityColors).map(([severity, color]) => {
            const count = point.by_severity[severity] || 0;
            return (React.createElement("div", { key: severity, className: "severity-segment", style: {
                    height: `${count * 5}px`,
                    backgroundColor: color,
                }, title: `${severity}: ${count}` }));
        })))))));
}
function StatusTrendsChart({ data, }) {
    const total = data.reduce((sum, d) => {
        return sum + Object.values(d.by_status).reduce((a, b) => a + b, 0);
    }, 0);
    const statusCounts = data.reduce((acc, d) => {
        Object.entries(d.by_status).forEach(([status, count]) => {
            acc[status] = (acc[status] || 0) + count;
        });
        return acc;
    }, {});
    return (React.createElement("div", { className: "status-pie-chart" }, Object.entries(statusCounts).map(([status, count]) => (React.createElement("div", { key: status, className: "status-segment" },
        React.createElement("div", { className: `status-color status-${status}` }),
        React.createElement("span", { className: "status-label" }, status),
        React.createElement("span", { className: "status-value" },
            count,
            " (",
            ((count / total) * 100).toFixed(1),
            "%)"))))));
}
function TimeRangeSelector({ value, onChange, }) {
    const presets = [
        { label: 'Last 7 Days', days: 7 },
        { label: 'Last 30 Days', days: 30 },
        { label: 'Last 90 Days', days: 90 },
        { label: 'Last Year', days: 365 },
    ];
    return (React.createElement("select", { onChange: (e) => {
            const days = parseInt(e.target.value);
            onChange({
                start: new Date(Date.now() - days * 24 * 60 * 60 * 1000),
                end: new Date(),
            });
        }, className: "time-range-select" }, presets.map((preset) => (React.createElement("option", { key: preset.days, value: preset.days }, preset.label)))));
}
function StatCard({ title, value, icon, trend, }) {
    return (React.createElement("div", { className: `stat-card trend-${trend}` },
        React.createElement("div", { className: "stat-icon" }, icon),
        React.createElement("div", { className: "stat-content" },
            React.createElement("div", { className: "stat-value" }, value),
            React.createElement("div", { className: "stat-title" }, title))));
}
function TrendCard({ title, value, icon }) {
    return (React.createElement("div", { className: "trend-card" },
        React.createElement("div", { className: "trend-icon" }, icon),
        React.createElement("div", { className: "trend-content" },
            React.createElement("div", { className: "trend-title" }, title),
            React.createElement("div", { className: "trend-value" }, value))));
}
function MetricCard({ title, value, color, }) {
    return (React.createElement("div", { className: `metric-card metric-${color}` },
        React.createElement("div", { className: "metric-value" }, value),
        React.createElement("div", { className: "metric-title" }, title)));
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
    return (React.createElement("span", { className: `risk-badge risk-${getColor(score)}` }, score.toFixed(0)));
}
function formatNumber(num) {
    return new Intl.NumberFormat('en-US').format(num);
}
function formatTimestamp(timestamp) {
    return new Date(timestamp).toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
    });
}
function findPeakActivity(trends) {
    if (trends.length === 0)
        return 'N/A';
    const peak = trends.reduce((max, trend) => trend.total > max.total ? trend : max);
    return `${new Date(peak.date).toLocaleDateString()} (${peak.total} events)`;
}
function calculateAnomalyRate(trends) {
    const total = trends.reduce((sum, t) => sum + t.total, 0);
    const anomalies = trends.reduce((sum, t) => sum + t.anomalies, 0);
    if (total === 0)
        return '0%';
    return `${((anomalies / total) * 100).toFixed(2)}%`;
}
function calculateHighRiskRate(trends) {
    const total = trends.reduce((sum, t) => sum + t.total, 0);
    const highRisk = trends.reduce((sum, t) => sum + t.high_risk, 0);
    if (total === 0)
        return '0%';
    return `${((highRisk / total) * 100).toFixed(2)}%`;
}
//# sourceMappingURL=AuditAnalytics.js.map