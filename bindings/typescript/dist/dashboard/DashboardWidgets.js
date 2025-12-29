import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useDashboard } from './DashboardLayout';
export const Widget = ({ config, children, onConfigChange, onRemove, onResize, editable = false, className = '', }) => {
    const [isExpanded, setIsExpanded] = useState(false);
    const [showMenu, setShowMenu] = useState(false);
    const [loading, setLoading] = useState({ isLoading: false });
    const [error, setError] = useState({ hasError: false });
    const [data, setData] = useState(null);
    const widgetRef = useRef(null);
    const { theme, accessibility, refreshData } = useDashboard();
    const fetchData = useCallback(async () => {
        if (!config.dataSource)
            return;
        setLoading({ isLoading: true, message: 'Loading widget data...' });
        setError({ hasError: false });
        try {
            const result = await fetchDataSource(config.dataSource);
            setData(result);
        }
        catch (err) {
            setError({
                hasError: true,
                message: err.message || 'Failed to load widget data',
                retry: fetchData,
            });
        }
        finally {
            setLoading({ isLoading: false });
        }
    }, [config.dataSource]);
    useEffect(() => {
        fetchData();
    }, [fetchData]);
    useEffect(() => {
        if (!config.autoRefresh || !config.refreshInterval)
            return;
        const interval = setInterval(fetchData, config.refreshInterval * 1000);
        return () => clearInterval(interval);
    }, [config.autoRefresh, config.refreshInterval, fetchData]);
    const handleRefresh = useCallback(() => {
        fetchData();
    }, [fetchData]);
    const handleToggleExpand = useCallback(() => {
        setIsExpanded((prev) => !prev);
    }, []);
    const handleRemove = useCallback(() => {
        if (onRemove) {
            onRemove(config.id);
        }
    }, [config.id, onRemove]);
    const handleResize = useCallback((newSize) => {
        if (onResize) {
            onResize(config.id, newSize);
        }
    }, [config.id, onResize]);
    useEffect(() => {
        if (!showMenu)
            return;
        const handleClickOutside = (event) => {
            if (widgetRef.current && !widgetRef.current.contains(event.target)) {
                setShowMenu(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, [showMenu]);
    return (React.createElement("div", { ref: widgetRef, className: `widget ${className} ${config.size} ${isExpanded ? 'expanded' : ''}`, style: {
            ...styles.widget,
            ...(isExpanded && styles.widgetExpanded),
            ...config.style,
        }, role: "region", "aria-label": config.title, "aria-describedby": `widget-desc-${config.id}` },
        React.createElement("div", { style: styles.header },
            React.createElement("div", { style: styles.headerLeft },
                React.createElement("h4", { style: styles.title, id: `widget-title-${config.id}` }, config.title),
                config.description && (React.createElement("p", { style: styles.description, id: `widget-desc-${config.id}` }, config.description))),
            React.createElement("div", { style: styles.headerRight },
                React.createElement("button", { onClick: handleRefresh, style: styles.iconButton, "aria-label": "Refresh widget", title: "Refresh", disabled: loading.isLoading }, "\u21BB"),
                React.createElement("button", { onClick: handleToggleExpand, style: styles.iconButton, "aria-label": isExpanded ? 'Collapse widget' : 'Expand widget', title: isExpanded ? 'Collapse' : 'Expand' }, isExpanded ? '⊖' : '⊕'),
                editable && (React.createElement("div", { style: styles.menuContainer },
                    React.createElement("button", { onClick: () => setShowMenu(!showMenu), style: styles.iconButton, "aria-label": "Widget menu", "aria-expanded": showMenu, "aria-haspopup": "true" }, "\u22EE"),
                    showMenu && (React.createElement("div", { style: styles.menu, role: "menu" },
                        React.createElement("button", { onClick: () => handleResize('small'), style: styles.menuItem, role: "menuitem" }, "Small"),
                        React.createElement("button", { onClick: () => handleResize('medium'), style: styles.menuItem, role: "menuitem" }, "Medium"),
                        React.createElement("button", { onClick: () => handleResize('large'), style: styles.menuItem, role: "menuitem" }, "Large"),
                        React.createElement("div", { style: styles.menuDivider, role: "separator" }),
                        React.createElement("button", { onClick: handleRemove, style: { ...styles.menuItem, ...styles.menuItemDanger }, role: "menuitem" }, "Remove"))))))),
        React.createElement("div", { style: styles.content },
            loading.isLoading && (React.createElement("div", { style: styles.loading, role: "status", "aria-live": "polite" },
                React.createElement("div", { style: styles.spinner }),
                React.createElement("p", null, loading.message || 'Loading...'))),
            error.hasError && (React.createElement("div", { style: styles.error, role: "alert" },
                React.createElement("p", { style: styles.errorMessage }, error.message),
                error.retry && (React.createElement("button", { onClick: error.retry, style: styles.retryButton }, "Retry")))),
            !loading.isLoading && !error.hasError && (React.createElement("div", { className: "widget-body", "aria-live": "polite" }, children || React.createElement(WidgetContent, { config: config, data: data }))))));
};
const WidgetContent = ({ config, data }) => {
    switch (config.type) {
        case 'metric':
            return React.createElement(MetricWidgetContent, { data: data, options: config.options });
        case 'chart':
            return React.createElement(ChartWidgetContent, { data: data, options: config.options });
        case 'table':
            return React.createElement(TableWidgetContent, { data: data, options: config.options });
        case 'feed':
            return React.createElement(FeedWidgetContent, { data: data, options: config.options });
        case 'custom':
            return React.createElement(CustomWidgetContent, { data: data, options: config.options });
        default:
            return React.createElement("div", null,
                "Widget type: ",
                config.type);
    }
};
const MetricWidgetContent = ({ data, options }) => {
    if (!data)
        return null;
    return (React.createElement("div", { style: styles.metricContent },
        React.createElement("div", { style: styles.metricValue }, data.value || 0),
        data.label && React.createElement("div", { style: styles.metricLabel }, data.label),
        data.change && (React.createElement("div", { style: {
                ...styles.metricChange,
                color: data.change >= 0 ? 'var(--color-success, #4caf50)' : 'var(--color-error, #f44336)',
            } },
            data.change >= 0 ? '↑' : '↓',
            " ",
            Math.abs(data.change),
            "%"))));
};
const ChartWidgetContent = ({ data, options }) => {
    if (!data || !data.datasets)
        return null;
    return (React.createElement("div", { style: styles.chartContent },
        React.createElement("canvas", { id: `chart-${Math.random()}`, width: "100%", height: "200" })));
};
const TableWidgetContent = ({ data, options }) => {
    if (!data || !Array.isArray(data.rows))
        return null;
    return (React.createElement("div", { style: styles.tableContent },
        React.createElement("table", { style: styles.table },
            React.createElement("thead", null,
                React.createElement("tr", null, data.columns?.map((col, index) => (React.createElement("th", { key: index, style: styles.tableHeader }, col))))),
            React.createElement("tbody", null, data.rows.map((row, rowIndex) => (React.createElement("tr", { key: rowIndex }, row.map((cell, cellIndex) => (React.createElement("td", { key: cellIndex, style: styles.tableCell }, cell))))))))));
};
const FeedWidgetContent = ({ data, options }) => {
    if (!data || !Array.isArray(data.items))
        return null;
    return (React.createElement("div", { style: styles.feedContent }, data.items.map((item, index) => (React.createElement("div", { key: index, style: styles.feedItem },
        React.createElement("div", { style: styles.feedItemTitle }, item.title),
        React.createElement("div", { style: styles.feedItemDescription }, item.description),
        React.createElement("div", { style: styles.feedItemTime }, item.timestamp))))));
};
const CustomWidgetContent = ({ data, options }) => {
    return (React.createElement("div", { style: styles.customContent },
        React.createElement("pre", null, JSON.stringify(data, null, 2))));
};
async function fetchDataSource(config) {
    switch (config.type) {
        case 'api':
            return fetchApiData(config);
        case 'websocket':
            return fetchWebSocketData(config);
        case 'static':
            return config.data || null;
        default:
            throw new Error(`Unsupported data source type: ${config.type}`);
    }
}
async function fetchApiData(config) {
    if (!config.url) {
        throw new Error('API URL is required');
    }
    const response = await fetch(config.url, {
        method: config.method || 'GET',
        headers: {
            'Content-Type': 'application/json',
            ...config.headers,
        },
        body: config.body ? JSON.stringify(config.body) : undefined,
    });
    if (!response.ok) {
        throw new Error(`API request failed: ${response.statusText}`);
    }
    return response.json();
}
async function fetchWebSocketData(config) {
    return { message: 'WebSocket data not implemented' };
}
export const WidgetGrid = ({ widgets, columns = 3, editable = false, onWidgetChange, className = '', }) => {
    const [widgetList, setWidgetList] = useState(widgets);
    const handleRemoveWidget = useCallback((widgetId) => {
        const updated = widgetList.filter((w) => w.id !== widgetId);
        setWidgetList(updated);
        if (onWidgetChange) {
            onWidgetChange(updated);
        }
    }, [widgetList, onWidgetChange]);
    const handleResizeWidget = useCallback((widgetId, size) => {
        const updated = widgetList.map((w) => w.id === widgetId ? { ...w, size } : w);
        setWidgetList(updated);
        if (onWidgetChange) {
            onWidgetChange(updated);
        }
    }, [widgetList, onWidgetChange]);
    useEffect(() => {
        setWidgetList(widgets);
    }, [widgets]);
    return (React.createElement("div", { className: `widget-grid ${className}`, style: {
            ...styles.grid,
            gridTemplateColumns: `repeat(${columns}, 1fr)`,
        } }, widgetList.map((widget) => (React.createElement("div", { key: widget.id, style: {
            gridColumn: `span ${widget.span.cols}`,
            gridRow: `span ${widget.span.rows}`,
        } },
        React.createElement(Widget, { config: widget, editable: editable, onRemove: handleRemoveWidget, onResize: handleResizeWidget }))))));
};
const styles = {
    widget: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        border: '1px solid var(--color-border, #e0e0e0)',
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        overflow: 'hidden',
        transition: 'box-shadow var(--animation-duration, 200ms)',
    },
    widgetExpanded: {
        position: 'fixed',
        top: '5%',
        left: '5%',
        right: '5%',
        bottom: '5%',
        zIndex: 1000,
        boxShadow: '0 8px 32px rgba(0, 0, 0, 0.2)',
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'flex-start',
        padding: '16px 20px',
        borderBottom: '1px solid var(--color-divider, #e0e0e0)',
    },
    headerLeft: {
        flex: 1,
    },
    headerRight: {
        display: 'flex',
        gap: 8,
    },
    title: {
        margin: 0,
        fontSize: 16,
        fontWeight: 600,
        color: 'var(--color-text, #333)',
    },
    description: {
        margin: '4px 0 0 0',
        fontSize: 12,
        color: 'var(--color-text-secondary, #666)',
    },
    iconButton: {
        width: 32,
        height: 32,
        border: 'none',
        backgroundColor: 'transparent',
        color: 'var(--color-text-secondary, #666)',
        cursor: 'pointer',
        borderRadius: 4,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: 18,
        transition: 'background-color var(--animation-duration, 200ms)',
    },
    menuContainer: {
        position: 'relative',
    },
    menu: {
        position: 'absolute',
        top: '100%',
        right: 0,
        marginTop: 4,
        backgroundColor: 'var(--color-surface, #fff)',
        border: '1px solid var(--color-border, #e0e0e0)',
        borderRadius: 4,
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.1)',
        minWidth: 120,
        zIndex: 1001,
    },
    menuItem: {
        width: '100%',
        padding: '8px 16px',
        border: 'none',
        backgroundColor: 'transparent',
        color: 'var(--color-text, #333)',
        cursor: 'pointer',
        fontSize: 14,
        textAlign: 'left',
        transition: 'background-color var(--animation-duration, 200ms)',
    },
    menuItemDanger: {
        color: 'var(--color-error, #f44336)',
    },
    menuDivider: {
        height: 1,
        backgroundColor: 'var(--color-divider, #e0e0e0)',
        margin: '4px 0',
    },
    content: {
        flex: 1,
        padding: 20,
        overflow: 'auto',
        position: 'relative',
    },
    loading: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100%',
        gap: 12,
    },
    spinner: {
        width: 40,
        height: 40,
        border: '4px solid var(--color-border, #e0e0e0)',
        borderTop: '4px solid var(--color-primary, #1976d2)',
        borderRadius: '50%',
        animation: 'spin 1s linear infinite',
    },
    error: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100%',
        gap: 12,
        padding: 20,
        textAlign: 'center',
    },
    errorMessage: {
        color: 'var(--color-error, #f44336)',
        margin: 0,
    },
    retryButton: {
        padding: '8px 16px',
        backgroundColor: 'var(--color-primary, #1976d2)',
        color: '#fff',
        border: 'none',
        borderRadius: 4,
        cursor: 'pointer',
        fontSize: 14,
        fontWeight: 500,
    },
    metricContent: {
        textAlign: 'center',
    },
    metricValue: {
        fontSize: 48,
        fontWeight: 700,
        color: 'var(--color-text, #333)',
        marginBottom: 8,
    },
    metricLabel: {
        fontSize: 16,
        color: 'var(--color-text-secondary, #666)',
        marginBottom: 8,
    },
    metricChange: {
        fontSize: 18,
        fontWeight: 600,
    },
    chartContent: {
        width: '100%',
        height: '100%',
    },
    tableContent: {
        overflowX: 'auto',
    },
    table: {
        width: '100%',
        borderCollapse: 'collapse',
        fontSize: 14,
    },
    tableHeader: {
        padding: '8px 12px',
        textAlign: 'left',
        fontWeight: 600,
        borderBottom: '2px solid var(--color-divider, #e0e0e0)',
        color: 'var(--color-text, #333)',
    },
    tableCell: {
        padding: '8px 12px',
        borderBottom: '1px solid var(--color-divider, #e0e0e0)',
        color: 'var(--color-text, #333)',
    },
    feedContent: {
        display: 'flex',
        flexDirection: 'column',
        gap: 12,
    },
    feedItem: {
        padding: 12,
        backgroundColor: 'var(--color-background, #f5f5f5)',
        borderRadius: 4,
    },
    feedItemTitle: {
        fontWeight: 600,
        marginBottom: 4,
        color: 'var(--color-text, #333)',
    },
    feedItemDescription: {
        fontSize: 13,
        color: 'var(--color-text-secondary, #666)',
        marginBottom: 4,
    },
    feedItemTime: {
        fontSize: 11,
        color: 'var(--color-text-secondary, #999)',
    },
    customContent: {
        fontSize: 12,
        fontFamily: 'monospace',
        overflow: 'auto',
    },
    grid: {
        display: 'grid',
        gap: 16,
        width: '100%',
    },
};
export default Widget;
//# sourceMappingURL=DashboardWidgets.js.map