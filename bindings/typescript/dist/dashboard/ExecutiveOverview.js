import React, { useState, useMemo } from 'react';
import { useDashboard } from './DashboardLayout';
export const ExecutiveOverview = ({ data, showPeriodSelector = true, onPeriodChange, showPrintButton = true, className = '', }) => {
    const [selectedPeriod, setSelectedPeriod] = useState(data.period);
    const { theme, accessibility } = useDashboard();
    const handlePeriodChange = (period) => {
        setSelectedPeriod(period);
        if (onPeriodChange) {
            onPeriodChange(period);
        }
    };
    const handlePrint = () => {
        window.print();
    };
    const healthScore = useMemo(() => {
        if (!data.performance)
            return 0;
        return data.performance.score;
    }, [data.performance]);
    const getHealthColor = (score) => {
        if (score >= 80)
            return 'var(--color-success, #4caf50)';
        if (score >= 60)
            return 'var(--color-warning, #ff9800)';
        return 'var(--color-error, #f44336)';
    };
    return (React.createElement("div", { className: `executive-overview ${className}`, style: styles.container, role: "article", "aria-label": "Executive overview" },
        React.createElement("div", { style: styles.header },
            React.createElement("div", { style: styles.headerLeft },
                React.createElement("h1", { style: styles.title }, "Executive Overview"),
                React.createElement("p", { style: styles.subtitle },
                    "Generated on ",
                    new Date(data.generatedAt).toLocaleDateString('en-US', {
                        weekday: 'long',
                        year: 'numeric',
                        month: 'long',
                        day: 'numeric',
                    }))),
            React.createElement("div", { style: styles.headerRight },
                showPeriodSelector && (React.createElement("select", { value: selectedPeriod, onChange: (e) => handlePeriodChange(e.target.value), style: styles.periodSelector, "aria-label": "Select time period" },
                    React.createElement("option", { value: "1h" }, "Last Hour"),
                    React.createElement("option", { value: "24h" }, "Last 24 Hours"),
                    React.createElement("option", { value: "7d" }, "Last 7 Days"),
                    React.createElement("option", { value: "30d" }, "Last 30 Days"),
                    React.createElement("option", { value: "90d" }, "Last 90 Days"),
                    React.createElement("option", { value: "1y" }, "Last Year"))),
                showPrintButton && (React.createElement("button", { onClick: handlePrint, style: styles.printButton, "aria-label": "Print overview" }, "\uD83D\uDDA8\uFE0F Print")))),
        React.createElement("div", { style: styles.healthSection },
            React.createElement("div", { style: styles.healthCard },
                React.createElement("h2", { style: styles.sectionTitle }, "Overall Health Score"),
                React.createElement("div", { style: {
                        ...styles.healthScore,
                        color: getHealthColor(healthScore),
                    }, role: "meter", "aria-valuenow": healthScore, "aria-valuemin": 0, "aria-valuemax": 100, "aria-label": `Health score: ${healthScore} out of 100` },
                    healthScore,
                    React.createElement("span", { style: styles.healthScoreUnit }, "/100")),
                React.createElement("div", { style: {
                        ...styles.healthBar,
                        width: `${healthScore}%`,
                        backgroundColor: getHealthColor(healthScore),
                    }, role: "presentation" }),
                React.createElement("p", { style: styles.healthTrend },
                    data.performance.trend === 'up' && '↑',
                    data.performance.trend === 'down' && '↓',
                    data.performance.trend === 'neutral' && '→',
                    ' ',
                    data.performance.trend === 'up' ? 'Improving' : data.performance.trend === 'down' ? 'Declining' : 'Stable'))),
        React.createElement("section", { style: styles.section, "aria-labelledby": "key-metrics-title" },
            React.createElement("h2", { style: styles.sectionTitle, id: "key-metrics-title" }, "Key Performance Indicators"),
            React.createElement("div", { style: styles.metricsGrid }, data.keyMetrics.map((metric) => (React.createElement(ExecutiveMetricCard, { key: metric.id, metric: metric }))))),
        React.createElement("section", { style: styles.section, "aria-labelledby": "revenue-title" },
            React.createElement("h2", { style: styles.sectionTitle, id: "revenue-title" }, "Revenue Summary"),
            React.createElement(RevenueSummary, { data: data.revenue })),
        React.createElement("section", { style: styles.section, "aria-labelledby": "initiatives-title" },
            React.createElement("h2", { style: styles.sectionTitle, id: "initiatives-title" }, "Strategic Initiatives"),
            React.createElement(InitiativesGrid, { initiatives: data.initiatives })),
        React.createElement("section", { style: styles.section, "aria-labelledby": "risks-title" },
            React.createElement("h2", { style: styles.sectionTitle, id: "risks-title" }, "Risk Dashboard"),
            React.createElement(RiskDashboard, { risks: data.risks })),
        React.createElement("section", { style: styles.section, "aria-labelledby": "highlights-title" },
            React.createElement("h2", { style: styles.sectionTitle, id: "highlights-title" }, "Key Highlights"),
            React.createElement(HighlightsList, { highlights: data.highlights })),
        React.createElement("section", { style: styles.section, "aria-labelledby": "recommendations-title" },
            React.createElement("h2", { style: styles.sectionTitle, id: "recommendations-title" }, "Strategic Recommendations"),
            React.createElement(RecommendationsList, { recommendations: data.recommendations }))));
};
const ExecutiveMetricCard = ({ metric }) => {
    const getTrendColor = (trend) => {
        if (trend === 'up')
            return 'var(--color-success, #4caf50)';
        if (trend === 'down')
            return 'var(--color-error, #f44336)';
        return 'var(--color-text-secondary, #666)';
    };
    return (React.createElement("div", { style: styles.metricCard },
        React.createElement("div", { style: styles.metricHeader },
            React.createElement("span", { style: styles.metricIcon }, metric.icon || '📊'),
            React.createElement("span", { style: styles.metricCategory }, metric.category)),
        React.createElement("h3", { style: styles.metricName }, metric.name),
        React.createElement("div", { style: styles.metricValue }, metric.formattedValue),
        metric.changePercent !== undefined && (React.createElement("div", { style: { ...styles.metricChange, color: getTrendColor(metric.trend) } },
            metric.trend === 'up' && '↑',
            metric.trend === 'down' && '↓',
            metric.trend === 'neutral' && '→',
            ' ',
            metric.changePercent >= 0 ? '+' : '',
            metric.changePercent.toFixed(1),
            "%"))));
};
const RevenueSummary = ({ data }) => {
    return (React.createElement("div", { style: styles.revenueSummary },
        React.createElement("div", { style: styles.revenueCard },
            React.createElement("h3", { style: styles.revenueLabel }, "Total Revenue"),
            React.createElement("div", { style: styles.revenueValue },
                "$",
                (data.total / 1000000).toFixed(2),
                "M"),
            React.createElement("div", { style: { ...styles.revenueGrowth, color: data.growth >= 0 ? 'var(--color-success, #4caf50)' : 'var(--color-error, #f44336)' } },
                data.growth >= 0 ? '↑' : '↓',
                " ",
                Math.abs(data.growth).toFixed(1),
                "% Growth")),
        data.target && (React.createElement("div", { style: styles.revenueCard },
            React.createElement("h3", { style: styles.revenueLabel }, "Target Attainment"),
            React.createElement("div", { style: styles.revenueValue },
                ((data.total / data.target) * 100).toFixed(1),
                "%"),
            React.createElement("div", { style: styles.revenueProgressBar },
                React.createElement("div", { style: {
                        ...styles.revenueProgressFill,
                        width: `${Math.min(100, (data.total / data.target) * 100)}%`,
                    } })))),
        React.createElement("div", { style: styles.revenueBreakdown },
            React.createElement("h3", { style: styles.revenueLabel }, "Revenue by Segment"),
            Object.entries(data.bySegment || {}).map(([segment, value]) => (React.createElement("div", { key: segment, style: styles.revenueSegment },
                React.createElement("span", { style: styles.revenueSegmentName }, segment),
                React.createElement("span", { style: styles.revenueSegmentValue },
                    "$",
                    (value / 1000000).toFixed(2),
                    "M")))))));
};
const InitiativesGrid = ({ initiatives }) => {
    const getStatusColor = (status) => {
        switch (status) {
            case 'completed':
                return 'var(--color-success, #4caf50)';
            case 'in-progress':
                return 'var(--color-info, #2196f3)';
            case 'on-hold':
                return 'var(--color-warning, #ff9800)';
            case 'cancelled':
                return 'var(--color-error, #f44336)';
            default:
                return 'var(--color-text-secondary, #666)';
        }
    };
    const getPriorityBadge = (priority) => {
        const colors = {
            critical: '#f44336',
            high: '#ff9800',
            medium: '#2196f3',
            low: '#4caf50',
        };
        return colors[priority] || '#666';
    };
    return (React.createElement("div", { style: styles.initiativesGrid }, initiatives.map((initiative) => (React.createElement("div", { key: initiative.id, style: styles.initiativeCard },
        React.createElement("div", { style: styles.initiativeHeader },
            React.createElement("h3", { style: styles.initiativeName }, initiative.name),
            React.createElement("span", { style: {
                    ...styles.priorityBadge,
                    backgroundColor: getPriorityBadge(initiative.priority),
                } }, initiative.priority)),
        React.createElement("p", { style: styles.initiativeDescription }, initiative.description),
        React.createElement("div", { style: styles.initiativeProgress },
            React.createElement("div", { style: styles.initiativeProgressHeader },
                React.createElement("span", { style: { ...styles.initiativeStatus, color: getStatusColor(initiative.status) } }, initiative.status),
                React.createElement("span", { style: styles.initiativeProgressText },
                    initiative.progress,
                    "%")),
            React.createElement("div", { style: styles.progressBar },
                React.createElement("div", { style: {
                        ...styles.progressFill,
                        width: `${initiative.progress}%`,
                        backgroundColor: getStatusColor(initiative.status),
                    } }))),
        React.createElement("div", { style: styles.initiativeFooter },
            React.createElement("span", { style: styles.initiativeOwner },
                "Owner: ",
                initiative.owner),
            React.createElement("span", { style: styles.initiativeDate },
                "Due: ",
                new Date(initiative.targetDate).toLocaleDateString())))))));
};
const RiskDashboard = ({ risks }) => {
    const getSeverityColor = (severity) => {
        switch (severity) {
            case 'critical':
                return '#d32f2f';
            case 'error':
                return '#f44336';
            case 'warning':
                return '#ff9800';
            default:
                return '#2196f3';
        }
    };
    const criticalRisks = risks.filter((r) => r.severity === 'critical' && !r.mitigated);
    const highRisks = risks.filter((r) => r.severity === 'error' && !r.mitigated);
    return (React.createElement("div", { style: styles.riskDashboard },
        React.createElement("div", { style: styles.riskSummary },
            React.createElement("div", { style: { ...styles.riskSummaryCard, borderColor: '#d32f2f' } },
                React.createElement("div", { style: styles.riskSummaryCount }, criticalRisks.length),
                React.createElement("div", { style: styles.riskSummaryLabel }, "Critical Risks")),
            React.createElement("div", { style: { ...styles.riskSummaryCard, borderColor: '#f44336' } },
                React.createElement("div", { style: styles.riskSummaryCount }, highRisks.length),
                React.createElement("div", { style: styles.riskSummaryLabel }, "High Risks")),
            React.createElement("div", { style: { ...styles.riskSummaryCard, borderColor: '#4caf50' } },
                React.createElement("div", { style: styles.riskSummaryCount }, risks.filter((r) => r.mitigated).length),
                React.createElement("div", { style: styles.riskSummaryLabel }, "Mitigated"))),
        React.createElement("div", { style: styles.riskList }, risks.map((risk) => (React.createElement("div", { key: risk.id, style: {
                ...styles.riskItem,
                borderLeftColor: getSeverityColor(risk.severity),
            } },
            React.createElement("div", { style: styles.riskItemHeader },
                React.createElement("h4", { style: styles.riskCategory }, risk.category),
                React.createElement("span", { style: {
                        ...styles.riskSeverity,
                        backgroundColor: getSeverityColor(risk.severity),
                    } }, risk.severity)),
            React.createElement("p", { style: styles.riskDescription }, risk.description),
            React.createElement("div", { style: styles.riskMetrics },
                React.createElement("div", { style: styles.riskMetric },
                    React.createElement("span", { style: styles.riskMetricLabel }, "Probability:"),
                    React.createElement("span", { style: styles.riskMetricValue },
                        risk.probability,
                        "%")),
                React.createElement("div", { style: styles.riskMetric },
                    React.createElement("span", { style: styles.riskMetricLabel }, "Impact:"),
                    React.createElement("span", { style: styles.riskMetricValue },
                        risk.impact,
                        "%")),
                React.createElement("div", { style: styles.riskMetric },
                    React.createElement("span", { style: styles.riskMetricLabel }, "Score:"),
                    React.createElement("span", { style: styles.riskMetricValue }, risk.score))),
            risk.mitigated && (React.createElement("div", { style: styles.riskMitigated }, "\u2713 Mitigated"))))))));
};
const HighlightsList = ({ highlights }) => {
    const getHighlightIcon = (type) => {
        switch (type) {
            case 'achievement':
                return '🏆';
            case 'milestone':
                return '🎯';
            case 'alert':
                return '⚠️';
            case 'insight':
                return '💡';
            default:
                return '📌';
        }
    };
    return (React.createElement("div", { style: styles.highlightsList }, highlights.map((highlight) => (React.createElement("div", { key: highlight.id, style: styles.highlightItem },
        React.createElement("span", { style: styles.highlightIcon }, highlight.icon || getHighlightIcon(highlight.type)),
        React.createElement("div", { style: styles.highlightContent },
            React.createElement("h4", { style: styles.highlightTitle }, highlight.title),
            React.createElement("p", { style: styles.highlightDescription }, highlight.description),
            React.createElement("span", { style: styles.highlightTime }, new Date(highlight.timestamp).toLocaleString())))))));
};
const RecommendationsList = ({ recommendations, }) => {
    const getPriorityColor = (priority) => {
        switch (priority) {
            case 'high':
                return '#f44336';
            case 'medium':
                return '#ff9800';
            default:
                return '#4caf50';
        }
    };
    return (React.createElement("div", { style: styles.recommendationsList }, recommendations.map((rec) => (React.createElement("div", { key: rec.id, style: styles.recommendationItem },
        React.createElement("div", { style: styles.recommendationHeader },
            React.createElement("h4", { style: styles.recommendationTitle }, rec.title),
            React.createElement("span", { style: {
                    ...styles.recommendationPriority,
                    backgroundColor: getPriorityColor(rec.priority),
                } },
                rec.priority,
                " priority")),
        React.createElement("p", { style: styles.recommendationDescription }, rec.description),
        React.createElement("div", { style: styles.recommendationMeta },
            React.createElement("span", { style: styles.recommendationCategory },
                "Category: ",
                rec.category),
            React.createElement("span", { style: styles.recommendationConfidence },
                "Confidence: ",
                rec.confidence,
                "%")),
        React.createElement("div", { style: styles.recommendationImpact },
            React.createElement("strong", null, "Expected Impact:"),
            " ",
            rec.impact),
        rec.actions && rec.actions.length > 0 && (React.createElement("div", { style: styles.recommendationActions },
            React.createElement("strong", null, "Action Items:"),
            React.createElement("ul", { style: styles.actionsList }, rec.actions.map((action, index) => (React.createElement("li", { key: index, style: styles.actionItem }, action)))))))))));
};
const styles = {
    container: {
        backgroundColor: 'var(--color-background, #f5f5f5)',
        padding: 32,
        minHeight: '100vh',
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'flex-start',
        marginBottom: 32,
        paddingBottom: 24,
        borderBottom: '2px solid var(--color-divider, #e0e0e0)',
    },
    headerLeft: {},
    headerRight: {
        display: 'flex',
        gap: 12,
        alignItems: 'center',
    },
    title: {
        margin: 0,
        fontSize: 32,
        fontWeight: 700,
        color: 'var(--color-text, #333)',
    },
    subtitle: {
        margin: '8px 0 0 0',
        fontSize: 14,
        color: 'var(--color-text-secondary, #666)',
    },
    periodSelector: {
        padding: '8px 16px',
        border: '1px solid var(--color-border, #e0e0e0)',
        borderRadius: 4,
        backgroundColor: 'var(--color-surface, #fff)',
        color: 'var(--color-text, #333)',
        fontSize: 14,
        cursor: 'pointer',
    },
    printButton: {
        padding: '8px 16px',
        backgroundColor: 'var(--color-primary, #1976d2)',
        color: '#fff',
        border: 'none',
        borderRadius: 4,
        cursor: 'pointer',
        fontSize: 14,
        fontWeight: 500,
    },
    healthSection: {
        marginBottom: 32,
    },
    healthCard: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        padding: 32,
        textAlign: 'center',
    },
    healthScore: {
        fontSize: 72,
        fontWeight: 700,
        margin: '16px 0',
    },
    healthScoreUnit: {
        fontSize: 32,
        opacity: 0.6,
    },
    healthBar: {
        height: 8,
        backgroundColor: 'var(--color-success, #4caf50)',
        borderRadius: 4,
        margin: '16px 0',
        transition: 'width 1s ease',
    },
    healthTrend: {
        fontSize: 18,
        fontWeight: 600,
        margin: 0,
    },
    section: {
        marginBottom: 32,
    },
    sectionTitle: {
        margin: '0 0 16px 0',
        fontSize: 24,
        fontWeight: 600,
        color: 'var(--color-text, #333)',
    },
    metricsGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
        gap: 16,
    },
    metricCard: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        padding: 20,
        border: '1px solid var(--color-border, #e0e0e0)',
    },
    metricHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: 12,
    },
    metricIcon: {
        fontSize: 24,
    },
    metricCategory: {
        fontSize: 11,
        padding: '2px 8px',
        backgroundColor: 'var(--color-background, #f5f5f5)',
        borderRadius: 12,
        fontWeight: 500,
    },
    metricName: {
        margin: '0 0 8px 0',
        fontSize: 14,
        fontWeight: 500,
        color: 'var(--color-text-secondary, #666)',
    },
    metricValue: {
        fontSize: 28,
        fontWeight: 700,
        color: 'var(--color-text, #333)',
        marginBottom: 8,
    },
    metricChange: {
        fontSize: 14,
        fontWeight: 600,
    },
    revenueSummary: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
        gap: 16,
    },
    revenueCard: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        padding: 24,
        border: '1px solid var(--color-border, #e0e0e0)',
    },
    revenueLabel: {
        margin: '0 0 12px 0',
        fontSize: 14,
        fontWeight: 500,
        color: 'var(--color-text-secondary, #666)',
    },
    revenueValue: {
        fontSize: 36,
        fontWeight: 700,
        color: 'var(--color-text, #333)',
        marginBottom: 8,
    },
    revenueGrowth: {
        fontSize: 16,
        fontWeight: 600,
    },
    revenueProgressBar: {
        height: 8,
        backgroundColor: 'var(--color-background, #f5f5f5)',
        borderRadius: 4,
        overflow: 'hidden',
    },
    revenueProgressFill: {
        height: '100%',
        backgroundColor: 'var(--color-success, #4caf50)',
        transition: 'width 1s ease',
    },
    revenueBreakdown: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        padding: 24,
        border: '1px solid var(--color-border, #e0e0e0)',
    },
    revenueSegment: {
        display: 'flex',
        justifyContent: 'space-between',
        padding: '8px 0',
        borderBottom: '1px solid var(--color-divider, #e0e0e0)',
    },
    revenueSegmentName: {
        fontSize: 14,
        color: 'var(--color-text, #333)',
    },
    revenueSegmentValue: {
        fontSize: 14,
        fontWeight: 600,
        color: 'var(--color-text, #333)',
    },
    initiativesGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))',
        gap: 16,
    },
    initiativeCard: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        padding: 20,
        border: '1px solid var(--color-border, #e0e0e0)',
    },
    initiativeHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'flex-start',
        marginBottom: 12,
    },
    initiativeName: {
        margin: 0,
        fontSize: 16,
        fontWeight: 600,
        color: 'var(--color-text, #333)',
        flex: 1,
    },
    priorityBadge: {
        fontSize: 10,
        padding: '4px 8px',
        borderRadius: 12,
        color: '#fff',
        fontWeight: 600,
        textTransform: 'uppercase',
    },
    initiativeDescription: {
        fontSize: 13,
        color: 'var(--color-text-secondary, #666)',
        marginBottom: 16,
    },
    initiativeProgress: {
        marginBottom: 12,
    },
    initiativeProgressHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        marginBottom: 8,
        fontSize: 12,
    },
    initiativeStatus: {
        fontWeight: 600,
        textTransform: 'capitalize',
    },
    initiativeProgressText: {
        fontWeight: 600,
    },
    progressBar: {
        height: 6,
        backgroundColor: 'var(--color-background, #f5f5f5)',
        borderRadius: 3,
        overflow: 'hidden',
    },
    progressFill: {
        height: '100%',
        transition: 'width 0.5s ease',
    },
    initiativeFooter: {
        display: 'flex',
        justifyContent: 'space-between',
        fontSize: 11,
        color: 'var(--color-text-secondary, #999)',
    },
    initiativeOwner: {},
    initiativeDate: {},
    riskDashboard: {},
    riskSummary: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
        gap: 16,
        marginBottom: 24,
    },
    riskSummaryCard: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        padding: 24,
        textAlign: 'center',
        borderLeft: '4px solid',
    },
    riskSummaryCount: {
        fontSize: 48,
        fontWeight: 700,
        color: 'var(--color-text, #333)',
    },
    riskSummaryLabel: {
        fontSize: 14,
        color: 'var(--color-text-secondary, #666)',
        marginTop: 8,
    },
    riskList: {
        display: 'flex',
        flexDirection: 'column',
        gap: 12,
    },
    riskItem: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        padding: 20,
        borderLeft: '4px solid',
    },
    riskItemHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: 12,
    },
    riskCategory: {
        margin: 0,
        fontSize: 16,
        fontWeight: 600,
        color: 'var(--color-text, #333)',
    },
    riskSeverity: {
        fontSize: 11,
        padding: '4px 12px',
        borderRadius: 12,
        color: '#fff',
        fontWeight: 600,
        textTransform: 'uppercase',
    },
    riskDescription: {
        fontSize: 14,
        color: 'var(--color-text, #333)',
        marginBottom: 12,
    },
    riskMetrics: {
        display: 'flex',
        gap: 24,
        marginBottom: 8,
    },
    riskMetric: {
        fontSize: 12,
    },
    riskMetricLabel: {
        color: 'var(--color-text-secondary, #666)',
        marginRight: 4,
    },
    riskMetricValue: {
        fontWeight: 600,
        color: 'var(--color-text, #333)',
    },
    riskMitigated: {
        display: 'inline-block',
        padding: '4px 12px',
        backgroundColor: '#e8f5e9',
        color: '#4caf50',
        borderRadius: 12,
        fontSize: 12,
        fontWeight: 600,
    },
    highlightsList: {
        display: 'flex',
        flexDirection: 'column',
        gap: 12,
    },
    highlightItem: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        padding: 20,
        display: 'flex',
        gap: 16,
        alignItems: 'flex-start',
    },
    highlightIcon: {
        fontSize: 32,
        lineHeight: 1,
    },
    highlightContent: {
        flex: 1,
    },
    highlightTitle: {
        margin: '0 0 8px 0',
        fontSize: 16,
        fontWeight: 600,
        color: 'var(--color-text, #333)',
    },
    highlightDescription: {
        margin: '0 0 8px 0',
        fontSize: 14,
        color: 'var(--color-text-secondary, #666)',
    },
    highlightTime: {
        fontSize: 12,
        color: 'var(--color-text-secondary, #999)',
    },
    recommendationsList: {
        display: 'flex',
        flexDirection: 'column',
        gap: 16,
    },
    recommendationItem: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        padding: 24,
        border: '1px solid var(--color-border, #e0e0e0)',
    },
    recommendationHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'flex-start',
        marginBottom: 12,
    },
    recommendationTitle: {
        margin: 0,
        fontSize: 18,
        fontWeight: 600,
        color: 'var(--color-text, #333)',
        flex: 1,
    },
    recommendationPriority: {
        fontSize: 11,
        padding: '4px 12px',
        borderRadius: 12,
        color: '#fff',
        fontWeight: 600,
        textTransform: 'uppercase',
    },
    recommendationDescription: {
        fontSize: 14,
        color: 'var(--color-text, #333)',
        marginBottom: 12,
        lineHeight: 1.6,
    },
    recommendationMeta: {
        display: 'flex',
        gap: 24,
        marginBottom: 12,
        fontSize: 13,
        color: 'var(--color-text-secondary, #666)',
    },
    recommendationCategory: {},
    recommendationConfidence: {},
    recommendationImpact: {
        fontSize: 14,
        color: 'var(--color-text, #333)',
        marginBottom: 12,
        padding: 12,
        backgroundColor: 'var(--color-background, #f5f5f5)',
        borderRadius: 4,
    },
    recommendationActions: {
        fontSize: 14,
    },
    actionsList: {
        margin: '8px 0 0 0',
        paddingLeft: 24,
    },
    actionItem: {
        marginBottom: 4,
        color: 'var(--color-text, #333)',
    },
};
export default ExecutiveOverview;
//# sourceMappingURL=ExecutiveOverview.js.map