import React, { useState, useEffect, useCallback, useMemo } from 'react';
export const ReportViewer = ({ reportId, definition, initialParameters = {}, onExecute, onDrillDown, onExport, autoRefresh = false, refreshInterval = 60000, showToolbar = true, showParameters = true, interactive = true, }) => {
    const [loading, setLoading] = useState(false);
    const [data, setData] = useState(null);
    const [parameters, setParameters] = useState(initialParameters);
    const [execution, setExecution] = useState(null);
    const [error, setError] = useState(null);
    const [currentPage, setCurrentPage] = useState(1);
    const [pageSize, setPageSize] = useState(50);
    const [sortConfig, setSortConfig] = useState([]);
    const [drillPath, setDrillPath] = useState([]);
    const [expandedRows, setExpandedRows] = useState(new Set());
    const executeReport = useCallback(async (params = parameters) => {
        if (!onExecute)
            return;
        setLoading(true);
        setError(null);
        const execStart = {
            id: generateExecutionId(),
            reportId,
            status: 'running',
            startTime: new Date(),
            parameters: params,
            executedBy: 'current-user',
            executionMode: 'interactive',
        };
        setExecution(execStart);
        try {
            const result = await onExecute(reportId, params);
            setData(result);
            setExecution({
                ...execStart,
                status: 'completed',
                endTime: new Date(),
                duration: Date.now() - execStart.startTime.getTime(),
                resultMetadata: {
                    rowCount: result.totalRows,
                    columnCount: result.columns.length,
                    dataSize: JSON.stringify(result).length,
                },
            });
        }
        catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Failed to execute report';
            setError(errorMessage);
            setExecution({
                ...execStart,
                status: 'failed',
                endTime: new Date(),
                duration: Date.now() - execStart.startTime.getTime(),
                error: {
                    code: 'EXECUTION_ERROR',
                    message: errorMessage,
                },
            });
        }
        finally {
            setLoading(false);
        }
    }, [reportId, parameters, onExecute]);
    useEffect(() => {
        executeReport();
    }, []);
    useEffect(() => {
        if (!autoRefresh || !refreshInterval)
            return;
        const interval = setInterval(() => {
            executeReport();
        }, refreshInterval);
        return () => clearInterval(interval);
    }, [autoRefresh, refreshInterval, executeReport]);
    const handleParameterChange = useCallback((name, value) => {
        setParameters((prev) => ({ ...prev, [name]: value }));
    }, []);
    const handleRunReport = useCallback(() => {
        setCurrentPage(1);
        executeReport(parameters);
    }, [parameters, executeReport]);
    const handleSort = useCallback((field) => {
        setSortConfig((prev) => {
            const existing = prev.find((s) => s.field === field);
            if (existing) {
                if (existing.direction === 'asc') {
                    return prev.map((s) => (s.field === field ? { ...s, direction: 'desc' } : s));
                }
                else {
                    return prev.filter((s) => s.field !== field);
                }
            }
            else {
                return [...prev, { field, direction: 'asc' }];
            }
        });
    }, []);
    const handleDrillDown = useCallback((rowData, config) => {
        if (!config || !config.enabled || !onDrillDown)
            return;
        const level = config.levels[0];
        if (!level)
            return;
        const filters = level.filters || [];
        Object.keys(rowData).forEach((key) => {
            filters.push({
                field: key,
                operator: 'eq',
                value: rowData[key],
            });
        });
        if (level.reportId) {
            setDrillPath((prev) => [
                ...prev,
                {
                    reportId,
                    reportName: definition?.name || 'Report',
                    filters,
                },
            ]);
            onDrillDown(level.reportId, filters);
        }
    }, [reportId, definition, onDrillDown]);
    const handleDrillBack = useCallback((index) => {
        const pathItem = drillPath[index];
        if (pathItem && onDrillDown) {
            onDrillDown(pathItem.reportId, pathItem.filters);
            setDrillPath((prev) => prev.slice(0, index));
        }
    }, [drillPath, onDrillDown]);
    const toggleRowExpansion = useCallback((rowIndex) => {
        setExpandedRows((prev) => {
            const newSet = new Set(prev);
            if (newSet.has(rowIndex)) {
                newSet.delete(rowIndex);
            }
            else {
                newSet.add(rowIndex);
            }
            return newSet;
        });
    }, []);
    const processedData = useMemo(() => {
        if (!data)
            return null;
        let rows = [...data.rows];
        if (sortConfig.length > 0) {
            rows.sort((a, b) => {
                for (const sort of sortConfig) {
                    const colIndex = data.columns.findIndex((c) => c.name === sort.field);
                    if (colIndex === -1)
                        continue;
                    const aVal = a[colIndex];
                    const bVal = b[colIndex];
                    if (aVal === bVal)
                        continue;
                    const comparison = aVal < bVal ? -1 : 1;
                    return sort.direction === 'asc' ? comparison : -comparison;
                }
                return 0;
            });
        }
        const start = (currentPage - 1) * pageSize;
        const end = start + pageSize;
        const paginatedRows = rows.slice(start, end);
        return {
            ...data,
            rows: paginatedRows,
            currentPage,
            totalPages: Math.ceil(rows.length / pageSize),
        };
    }, [data, sortConfig, currentPage, pageSize]);
    const renderParameterControls = () => {
        if (!definition?.parameters || definition.parameters.length === 0)
            return null;
        return (React.createElement("div", { style: styles.parametersPanel },
            React.createElement("h3", { style: styles.parametersPanelTitle }, "Parameters"),
            React.createElement("div", { style: styles.parametersGrid }, definition.parameters.map((param) => (React.createElement("div", { key: param.name, style: styles.parameterControl },
                React.createElement("label", { style: styles.parameterLabel },
                    param.displayName,
                    param.required && React.createElement("span", { style: styles.required }, "*")),
                renderParameterInput(param, parameters[param.name], handleParameterChange),
                param.ui?.helpText && (React.createElement("span", { style: styles.helpText }, param.ui.helpText)))))),
            React.createElement("button", { onClick: handleRunReport, style: styles.runButton, disabled: loading }, loading ? 'Running...' : '▶ Run Report')));
    };
    const renderDrillPath = () => {
        if (drillPath.length === 0)
            return null;
        return (React.createElement("div", { style: styles.drillPath },
            React.createElement("button", { onClick: () => handleDrillBack(-1), style: styles.drillBackButton },
                "\u2190 Back to ",
                definition?.name),
            React.createElement("div", { style: styles.breadcrumb },
                drillPath.map((item, index) => (React.createElement(React.Fragment, { key: index },
                    React.createElement("button", { onClick: () => handleDrillBack(index), style: styles.breadcrumbItem }, item.reportName),
                    React.createElement("span", { style: styles.breadcrumbSeparator }, "/")))),
                React.createElement("span", { style: styles.breadcrumbCurrent }, "Current"))));
    };
    const renderTable = () => {
        if (!processedData)
            return null;
        return (React.createElement("div", { style: styles.tableContainer },
            React.createElement("table", { style: styles.table },
                React.createElement("thead", null,
                    React.createElement("tr", null,
                        processedData.columns.map((column) => (React.createElement("th", { key: column.name, style: styles.tableHeader, onClick: () => interactive && handleSort(column.name) },
                            React.createElement("div", { style: styles.headerContent },
                                React.createElement("span", null, column.displayName),
                                interactive && (React.createElement("span", { style: styles.sortIcon }, getSortIcon(column.name, sortConfig))))))),
                        interactive && React.createElement("th", { style: styles.tableHeader }, "Actions"))),
                React.createElement("tbody", null, processedData.rows.map((row, rowIndex) => (React.createElement(React.Fragment, { key: rowIndex },
                    React.createElement("tr", { style: {
                            ...styles.tableRow,
                            backgroundColor: rowIndex % 2 === 0 ? '#ffffff' : '#f8fafc',
                        } },
                        row.map((cell, cellIndex) => (React.createElement("td", { key: cellIndex, style: styles.tableCell }, formatCellValue(cell, processedData.columns[cellIndex])))),
                        interactive && (React.createElement("td", { style: styles.tableCell },
                            React.createElement("button", { onClick: () => toggleRowExpansion(rowIndex), style: styles.actionButton, title: "Expand/Collapse" }, expandedRows.has(rowIndex) ? '−' : '+'),
                            definition?.layout.sections.some((s) => s.config?.drillDown?.enabled) && (React.createElement("button", { onClick: () => {
                                    const rowData = processedData.columns.reduce((acc, col, idx) => ({ ...acc, [col.name]: row[idx] }), {});
                                    handleDrillDown(rowData, definition.layout.sections.find((s) => s.config?.drillDown)
                                        ?.config?.drillDown);
                                }, style: styles.actionButton, title: "Drill Down" }, "\uD83D\uDD0D"))))),
                    expandedRows.has(rowIndex) && (React.createElement("tr", null,
                        React.createElement("td", { colSpan: processedData.columns.length + (interactive ? 1 : 0) },
                            React.createElement("div", { style: styles.expandedContent },
                                React.createElement("pre", null, JSON.stringify(row, null, 2)))))))))))));
    };
    const renderPagination = () => {
        if (!processedData || processedData.totalPages <= 1)
            return null;
        return (React.createElement("div", { style: styles.pagination },
            React.createElement("div", { style: styles.paginationInfo },
                "Showing ",
                ((currentPage - 1) * pageSize) + 1,
                " to",
                ' ',
                Math.min(currentPage * pageSize, data.totalRows),
                " of ",
                data.totalRows,
                " rows"),
            React.createElement("div", { style: styles.paginationControls },
                React.createElement("button", { onClick: () => setCurrentPage(1), disabled: currentPage === 1, style: styles.paginationButton }, "\u00AB"),
                React.createElement("button", { onClick: () => setCurrentPage((p) => Math.max(1, p - 1)), disabled: currentPage === 1, style: styles.paginationButton }, "\u2039"),
                React.createElement("span", { style: styles.pageNumber },
                    "Page ",
                    currentPage,
                    " of ",
                    processedData.totalPages),
                React.createElement("button", { onClick: () => setCurrentPage((p) => Math.min(processedData.totalPages, p + 1)), disabled: currentPage === processedData.totalPages, style: styles.paginationButton }, "\u203A"),
                React.createElement("button", { onClick: () => setCurrentPage(processedData.totalPages), disabled: currentPage === processedData.totalPages, style: styles.paginationButton }, "\u00BB"),
                React.createElement("select", { value: pageSize, onChange: (e) => {
                        setPageSize(Number(e.target.value));
                        setCurrentPage(1);
                    }, style: styles.pageSizeSelect },
                    React.createElement("option", { value: 25 }, "25 rows"),
                    React.createElement("option", { value: 50 }, "50 rows"),
                    React.createElement("option", { value: 100 }, "100 rows"),
                    React.createElement("option", { value: 250 }, "250 rows")))));
    };
    return (React.createElement("div", { style: styles.container },
        showToolbar && (React.createElement("div", { style: styles.toolbar },
            React.createElement("div", { style: styles.toolbarLeft },
                React.createElement("h2", { style: styles.reportTitle }, definition?.name || 'Report'),
                execution && (React.createElement("span", { style: styles.executionBadge },
                    execution.status === 'completed' && `✓ ${execution.duration}ms`,
                    execution.status === 'running' && '⟳ Running...',
                    execution.status === 'failed' && '✗ Failed'))),
            React.createElement("div", { style: styles.toolbarRight },
                React.createElement("button", { onClick: () => executeReport(), style: styles.toolbarButton, disabled: loading }, "\uD83D\uDD04 Refresh"),
                onExport && (React.createElement("div", { style: styles.exportDropdown },
                    React.createElement("button", { style: styles.toolbarButton }, "\u2B07 Export"),
                    React.createElement("div", { style: styles.exportMenu },
                        React.createElement("button", { onClick: () => onExport('pdf') }, "PDF"),
                        React.createElement("button", { onClick: () => onExport('excel') }, "Excel"),
                        React.createElement("button", { onClick: () => onExport('csv') }, "CSV"))))))),
        showParameters && renderParameterControls(),
        renderDrillPath(),
        React.createElement("div", { style: styles.content },
            loading && (React.createElement("div", { style: styles.loadingOverlay },
                React.createElement("div", { style: styles.spinner }, "Loading..."))),
            error && (React.createElement("div", { style: styles.errorContainer },
                React.createElement("div", { style: styles.errorIcon }, "\u26A0\uFE0F"),
                React.createElement("div", { style: styles.errorMessage }, error),
                React.createElement("button", { onClick: () => executeReport(), style: styles.retryButton }, "Retry"))),
            !loading && !error && data && (React.createElement(React.Fragment, null,
                renderTable(),
                renderPagination())))));
};
function generateExecutionId() {
    return `exec-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}
function renderParameterInput(param, value, onChange) {
    switch (param.ui?.inputType || 'text') {
        case 'select':
            return (React.createElement("select", { value: value || param.defaultValue || '', onChange: (e) => onChange(param.name, e.target.value), style: styles.parameterInput },
                React.createElement("option", { value: "" }, "Select..."),
                param.allowedValues?.map((val) => (React.createElement("option", { key: String(val), value: String(val) }, String(val))))));
        case 'date':
            return (React.createElement("input", { type: "date", value: value || param.defaultValue || '', onChange: (e) => onChange(param.name, e.target.value), style: styles.parameterInput }));
        case 'number':
            return (React.createElement("input", { type: "number", value: value || param.defaultValue || '', onChange: (e) => onChange(param.name, Number(e.target.value)), style: styles.parameterInput }));
        case 'checkbox':
            return (React.createElement("input", { type: "checkbox", checked: value || param.defaultValue || false, onChange: (e) => onChange(param.name, e.target.checked), style: styles.parameterCheckbox }));
        default:
            return (React.createElement("input", { type: "text", value: value || param.defaultValue || '', onChange: (e) => onChange(param.name, e.target.value), placeholder: param.ui?.placeholder, style: styles.parameterInput }));
    }
}
function getSortIcon(field, sortConfig) {
    const sort = sortConfig.find((s) => s.field === field);
    if (!sort)
        return '↕';
    return sort.direction === 'asc' ? '↑' : '↓';
}
function formatCellValue(value, column) {
    if (value === null || value === undefined)
        return '';
    if (column.format) {
        if (column.format.includes('%')) {
            return `${(Number(value) * 100).toFixed(2)}%`;
        }
        if (column.format.includes('$')) {
            return `$${Number(value).toLocaleString()}`;
        }
    }
    if (column.dataType === 'date') {
        return new Date(value).toLocaleDateString();
    }
    if (column.dataType === 'number') {
        return Number(value).toLocaleString();
    }
    return String(value);
}
const styles = {
    container: {
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        backgroundColor: '#f8fafc',
        fontFamily: 'Inter, system-ui, sans-serif',
    },
    toolbar: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '16px',
        backgroundColor: '#ffffff',
        borderBottom: '1px solid #e2e8f0',
    },
    toolbarLeft: {
        display: 'flex',
        alignItems: 'center',
        gap: '12px',
    },
    toolbarRight: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
    },
    reportTitle: {
        fontSize: '20px',
        fontWeight: 600,
        margin: 0,
        color: '#1e293b',
    },
    executionBadge: {
        fontSize: '12px',
        padding: '4px 8px',
        backgroundColor: '#dbeafe',
        color: '#1e40af',
        borderRadius: '12px',
        fontWeight: 500,
    },
    toolbarButton: {
        padding: '8px 16px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '14px',
        fontWeight: 500,
    },
    exportDropdown: {
        position: 'relative',
    },
    exportMenu: {
        display: 'none',
        position: 'absolute',
        top: '100%',
        right: 0,
        marginTop: '4px',
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
    },
    parametersPanel: {
        padding: '16px',
        backgroundColor: '#ffffff',
        borderBottom: '1px solid #e2e8f0',
    },
    parametersPanelTitle: {
        fontSize: '14px',
        fontWeight: 600,
        marginBottom: '12px',
        color: '#1e293b',
    },
    parametersGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
        gap: '12px',
        marginBottom: '12px',
    },
    parameterControl: {
        display: 'flex',
        flexDirection: 'column',
        gap: '4px',
    },
    parameterLabel: {
        fontSize: '12px',
        fontWeight: 500,
        color: '#475569',
    },
    required: {
        color: '#ef4444',
        marginLeft: '2px',
    },
    parameterInput: {
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '13px',
    },
    parameterCheckbox: {
        width: '18px',
        height: '18px',
    },
    helpText: {
        fontSize: '11px',
        color: '#64748b',
    },
    runButton: {
        padding: '8px 16px',
        backgroundColor: '#2563eb',
        color: '#ffffff',
        border: 'none',
        borderRadius: '6px',
        fontSize: '14px',
        fontWeight: 500,
        cursor: 'pointer',
    },
    drillPath: {
        padding: '12px 16px',
        backgroundColor: '#f8fafc',
        borderBottom: '1px solid #e2e8f0',
    },
    drillBackButton: {
        padding: '4px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '13px',
        marginBottom: '8px',
    },
    breadcrumb: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        fontSize: '13px',
    },
    breadcrumbItem: {
        background: 'none',
        border: 'none',
        color: '#2563eb',
        cursor: 'pointer',
        textDecoration: 'underline',
    },
    breadcrumbSeparator: {
        color: '#94a3b8',
    },
    breadcrumbCurrent: {
        color: '#475569',
        fontWeight: 500,
    },
    content: {
        flex: 1,
        overflow: 'auto',
        position: 'relative',
    },
    loadingOverlay: {
        position: 'absolute',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        backgroundColor: 'rgba(255, 255, 255, 0.8)',
        zIndex: 10,
    },
    spinner: {
        fontSize: '18px',
        fontWeight: 500,
        color: '#64748b',
    },
    errorContainer: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '48px',
        gap: '16px',
    },
    errorIcon: {
        fontSize: '48px',
    },
    errorMessage: {
        fontSize: '16px',
        color: '#ef4444',
        textAlign: 'center',
    },
    retryButton: {
        padding: '8px 16px',
        backgroundColor: '#2563eb',
        color: '#ffffff',
        border: 'none',
        borderRadius: '6px',
        fontSize: '14px',
        cursor: 'pointer',
    },
    tableContainer: {
        overflowX: 'auto',
    },
    table: {
        width: '100%',
        borderCollapse: 'collapse',
        backgroundColor: '#ffffff',
    },
    tableHeader: {
        padding: '12px',
        textAlign: 'left',
        borderBottom: '2px solid #e2e8f0',
        backgroundColor: '#f8fafc',
        fontWeight: 600,
        fontSize: '13px',
        color: '#475569',
        cursor: 'pointer',
        userSelect: 'none',
    },
    headerContent: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
    },
    sortIcon: {
        marginLeft: '4px',
        fontSize: '12px',
        color: '#94a3b8',
    },
    tableRow: {
        transition: 'background-color 0.2s',
    },
    tableCell: {
        padding: '12px',
        borderBottom: '1px solid #e2e8f0',
        fontSize: '13px',
        color: '#475569',
    },
    actionButton: {
        padding: '4px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '12px',
        marginRight: '4px',
    },
    expandedContent: {
        padding: '16px',
        backgroundColor: '#f8fafc',
        fontSize: '12px',
        fontFamily: 'monospace',
    },
    pagination: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '16px',
        backgroundColor: '#ffffff',
        borderTop: '1px solid #e2e8f0',
    },
    paginationInfo: {
        fontSize: '13px',
        color: '#64748b',
    },
    paginationControls: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
    },
    paginationButton: {
        padding: '6px 12px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '14px',
    },
    pageNumber: {
        fontSize: '13px',
        color: '#475569',
        padding: '0 8px',
    },
    pageSizeSelect: {
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '13px',
        cursor: 'pointer',
    },
};
export default ReportViewer;
//# sourceMappingURL=ReportViewer.js.map