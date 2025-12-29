import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { useDashboard } from './DashboardLayout';
export const MetricCard = ({ metric, showSparkline = true, showComparison = true, showProgress = false, size = 'medium', onClick, className = '', isLoading = false, }) => {
    const { theme, accessibility } = useDashboard();
    const [isHovered, setIsHovered] = useState(false);
    const getTrendIcon = (trend) => {
        switch (trend) {
            case 'up':
                return '↑';
            case 'down':
                return '↓';
            default:
                return '→';
        }
    };
    const getTrendColor = (trend) => {
        switch (trend) {
            case 'up':
                return 'var(--color-success, #4caf50)';
            case 'down':
                return 'var(--color-error, #f44336)';
            default:
                return 'var(--color-text-secondary, #666)';
        }
    };
    const formatChange = () => {
        if (!metric.changePercent)
            return '';
        const sign = metric.changePercent >= 0 ? '+' : '';
        return `${sign}${metric.changePercent.toFixed(1)}%`;
    };
    const renderSparkline = () => {
        if (!showSparkline || !metric.history || metric.history.length === 0) {
            return null;
        }
        const width = 100;
        const height = 30;
        const values = metric.history.map((p) => p.y);
        const min = Math.min(...values);
        const max = Math.max(...values);
        const range = max - min || 1;
        const points = metric.history
            .map((point, i) => {
            const x = (i / (metric.history.length - 1)) * width;
            const y = height - ((point.y - min) / range) * height;
            return `${x},${y}`;
        })
            .join(' ');
        return (React.createElement("svg", { width: width, height: height, style: styles.sparkline, role: "img", "aria-label": `Trend chart for ${metric.name}` },
            React.createElement("polyline", { points: points, fill: "none", stroke: getTrendColor(metric.trend), strokeWidth: "2", strokeLinecap: "round", strokeLinejoin: "round" })));
    };
    const renderProgress = () => {
        if (!showProgress || metric.progress === undefined) {
            return null;
        }
        return (React.createElement("div", { style: styles.progressContainer },
            React.createElement("div", { style: styles.progressBar },
                React.createElement("div", { style: {
                        ...styles.progressFill,
                        width: `${Math.min(100, Math.max(0, metric.progress))}%`,
                        backgroundColor: getTrendColor(metric.trend),
                    }, role: "progressbar", "aria-valuenow": metric.progress, "aria-valuemin": 0, "aria-valuemax": 100, "aria-label": `Progress: ${metric.progress}%` })),
            React.createElement("span", { style: styles.progressText },
                metric.progress.toFixed(0),
                "%")));
    };
    if (isLoading) {
        return (React.createElement("div", { className: `metric-card skeleton ${className}`, style: { ...styles.card, ...styles[`card${capitalize(size)}`] } },
            React.createElement("div", { style: styles.skeleton })));
    }
    return (React.createElement("div", { className: `metric-card ${className} ${size}`, style: {
            ...styles.card,
            ...styles[`card${capitalize(size)}`],
            ...(isHovered && styles.cardHover),
            ...(onClick && { cursor: 'pointer' }),
            ...(metric.color && { borderLeftColor: metric.color, borderLeftWidth: 4 }),
        }, onClick: () => onClick?.(metric), onMouseEnter: () => setIsHovered(true), onMouseLeave: () => setIsHovered(false), onFocus: () => setIsHovered(true), onBlur: () => setIsHovered(false), role: onClick ? 'button' : 'article', tabIndex: accessibility.keyboardNavigation && onClick ? 0 : undefined, "aria-label": `${metric.name}: ${metric.formattedValue}` },
        React.createElement("div", { style: styles.header },
            React.createElement("div", { style: styles.headerLeft },
                metric.icon && React.createElement("span", { style: styles.icon }, metric.icon),
                React.createElement("h3", { style: styles.title }, metric.name)),
            metric.category && (React.createElement("span", { style: styles.category, "aria-label": `Category: ${metric.category}` }, metric.category))),
        React.createElement("div", { style: styles.valueContainer },
            React.createElement("div", { style: styles.value, "aria-label": `Value: ${metric.formattedValue}` }, metric.formattedValue),
            metric.unit && (React.createElement("span", { style: styles.unit, "aria-label": `Unit: ${metric.unit}` }, metric.unit))),
        showComparison && metric.changePercent !== undefined && (React.createElement("div", { style: {
                ...styles.comparison,
                color: getTrendColor(metric.trend),
            }, "aria-label": `Change: ${formatChange()}` },
            React.createElement("span", { style: styles.trendIcon }, getTrendIcon(metric.trend)),
            React.createElement("span", { style: styles.changeText }, formatChange()),
            metric.previousValue !== undefined && (React.createElement("span", { style: styles.previousValue },
                "from ",
                metric.previousValue)))),
        renderProgress(),
        renderSparkline(),
        metric.target !== undefined && (React.createElement("div", { style: styles.target, "aria-label": `Target: ${metric.target}` },
            "Target: ",
            metric.target,
            " ",
            metric.unit)),
        React.createElement("div", { style: styles.footer },
            React.createElement("span", { style: styles.lastUpdated, "aria-label": `Last updated: ${metric.lastUpdated}` },
                "Updated: ",
                new Date(metric.lastUpdated).toLocaleString()))));
};
export const MetricsGrid = ({ metrics: initialMetrics, columns = { xs: 1, sm: 2, md: 3, lg: 4, xl: 4 }, showSparklines = true, showComparisons = true, showProgress = false, cardSize = 'medium', onMetricClick, refreshInterval, onRefresh, className = '', }) => {
    const [metrics, setMetrics] = useState(initialMetrics);
    const [loading, setLoading] = useState({ isLoading: false });
    const [screenSize, setScreenSize] = useState('lg');
    useEffect(() => {
        const handleResize = () => {
            const width = window.innerWidth;
            if (width < 576)
                setScreenSize('xs');
            else if (width < 768)
                setScreenSize('sm');
            else if (width < 992)
                setScreenSize('md');
            else if (width < 1200)
                setScreenSize('lg');
            else
                setScreenSize('xl');
        };
        handleResize();
        window.addEventListener('resize', handleResize);
        return () => window.removeEventListener('resize', handleResize);
    }, []);
    const refreshMetrics = useCallback(async () => {
        if (!onRefresh)
            return;
        setLoading({ isLoading: true, message: 'Refreshing metrics...' });
        try {
            const newMetrics = await onRefresh();
            setMetrics(newMetrics);
        }
        catch (error) {
            console.error('Failed to refresh metrics:', error);
        }
        finally {
            setLoading({ isLoading: false });
        }
    }, [onRefresh]);
    useEffect(() => {
        if (!refreshInterval || !onRefresh)
            return;
        const interval = setInterval(refreshMetrics, refreshInterval);
        return () => clearInterval(interval);
    }, [refreshInterval, refreshMetrics, onRefresh]);
    useEffect(() => {
        setMetrics(initialMetrics);
    }, [initialMetrics]);
    const gridColumns = useMemo(() => {
        return columns[screenSize] || 4;
    }, [columns, screenSize]);
    return (React.createElement("div", { className: `metrics-grid ${className}`, style: {
            ...styles.grid,
            gridTemplateColumns: `repeat(${gridColumns}, 1fr)`,
        }, role: "region", "aria-label": "Metrics dashboard" }, metrics.map((metric) => (React.createElement(MetricCard, { key: metric.id, metric: metric, showSparkline: showSparklines, showComparison: showComparisons, showProgress: showProgress, size: cardSize, onClick: onMetricClick, isLoading: loading.isLoading })))));
};
function capitalize(str) {
    return str.charAt(0).toUpperCase() + str.slice(1);
}
const styles = {
    grid: {
        display: 'grid',
        gap: 16,
        width: '100%',
    },
    card: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        border: '1px solid var(--color-border, #e0e0e0)',
        padding: 20,
        transition: 'transform var(--animation-duration, 200ms), box-shadow var(--animation-duration, 200ms)',
        position: 'relative',
        overflow: 'hidden',
    },
    cardHover: {
        transform: 'translateY(-2px)',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.1)',
    },
    cardSmall: {
        padding: 12,
    },
    cardMedium: {
        padding: 20,
    },
    cardLarge: {
        padding: 24,
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'flex-start',
        marginBottom: 12,
    },
    headerLeft: {
        display: 'flex',
        alignItems: 'center',
        gap: 8,
    },
    icon: {
        fontSize: 24,
        lineHeight: 1,
    },
    title: {
        margin: 0,
        fontSize: 14,
        fontWeight: 500,
        color: 'var(--color-text-secondary, #666)',
        lineHeight: 1.4,
    },
    category: {
        fontSize: 11,
        padding: '2px 8px',
        backgroundColor: 'var(--color-background, #f5f5f5)',
        borderRadius: 12,
        color: 'var(--color-text-secondary, #666)',
        fontWeight: 500,
    },
    valueContainer: {
        display: 'flex',
        alignItems: 'baseline',
        gap: 8,
        marginBottom: 8,
    },
    value: {
        fontSize: 32,
        fontWeight: 700,
        color: 'var(--color-text, #333)',
        lineHeight: 1.2,
    },
    unit: {
        fontSize: 14,
        color: 'var(--color-text-secondary, #666)',
        fontWeight: 500,
    },
    comparison: {
        display: 'flex',
        alignItems: 'center',
        gap: 4,
        fontSize: 14,
        fontWeight: 600,
        marginBottom: 12,
    },
    trendIcon: {
        fontSize: 16,
        lineHeight: 1,
    },
    changeText: {
        fontWeight: 600,
    },
    previousValue: {
        fontSize: 12,
        marginLeft: 4,
        color: 'var(--color-text-secondary, #666)',
        fontWeight: 400,
    },
    progressContainer: {
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        marginBottom: 12,
    },
    progressBar: {
        flex: 1,
        height: 6,
        backgroundColor: 'var(--color-background, #f5f5f5)',
        borderRadius: 3,
        overflow: 'hidden',
    },
    progressFill: {
        height: '100%',
        borderRadius: 3,
        transition: 'width var(--animation-duration, 200ms)',
    },
    progressText: {
        fontSize: 12,
        fontWeight: 600,
        color: 'var(--color-text-secondary, #666)',
        minWidth: 40,
        textAlign: 'right',
    },
    sparkline: {
        marginTop: 8,
        marginBottom: 8,
        opacity: 0.8,
    },
    target: {
        fontSize: 12,
        color: 'var(--color-text-secondary, #666)',
        marginTop: 8,
    },
    footer: {
        marginTop: 12,
        paddingTop: 12,
        borderTop: '1px solid var(--color-divider, #e0e0e0)',
    },
    lastUpdated: {
        fontSize: 11,
        color: 'var(--color-text-secondary, #999)',
    },
    skeleton: {
        width: '100%',
        height: 120,
        backgroundColor: 'var(--color-background, #f5f5f5)',
        borderRadius: 4,
        animation: 'pulse 1.5s ease-in-out infinite',
    },
};
export default MetricsGrid;
//# sourceMappingURL=DashboardMetrics.js.map