import React, { useState, useCallback, useMemo } from 'react';
export const ReportDashboard = ({ reports, executions = [], onCreateReport, onEditReport, onDeleteReport, onDuplicateReport, onExecuteReport, onScheduleReport, onViewReport, onExportReport, }) => {
    const [searchTerm, setSearchTerm] = useState('');
    const [statusFilter, setStatusFilter] = useState('all');
    const [categoryFilter, setCategoryFilter] = useState('all');
    const [sortBy, setSortBy] = useState('updated');
    const [viewMode, setViewMode] = useState('grid');
    const [selectedReports, setSelectedReports] = useState(new Set());
    const categories = useMemo(() => ['all', ...new Set(reports.map((r) => r.metadata.category).filter(Boolean))], [reports]);
    const filteredReports = useMemo(() => {
        return reports
            .filter((report) => {
            const matchesSearch = report.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                report.description?.toLowerCase().includes(searchTerm.toLowerCase()) ||
                report.metadata.tags?.some((tag) => tag.toLowerCase().includes(searchTerm.toLowerCase()));
            const matchesStatus = statusFilter === 'all' || report.status === statusFilter;
            const matchesCategory = categoryFilter === 'all' || report.metadata.category === categoryFilter;
            return matchesSearch && matchesStatus && matchesCategory;
        })
            .sort((a, b) => {
            switch (sortBy) {
                case 'name':
                    return a.name.localeCompare(b.name);
                case 'updated':
                    return new Date(b.metadata.updatedAt).getTime() - new Date(a.metadata.updatedAt).getTime();
                case 'created':
                    return new Date(b.metadata.createdAt).getTime() - new Date(a.metadata.createdAt).getTime();
                default:
                    return 0;
            }
        });
    }, [reports, searchTerm, statusFilter, categoryFilter, sortBy]);
    const toggleReportSelection = useCallback((reportId) => {
        setSelectedReports((prev) => {
            const newSet = new Set(prev);
            if (newSet.has(reportId)) {
                newSet.delete(reportId);
            }
            else {
                newSet.add(reportId);
            }
            return newSet;
        });
    }, []);
    const toggleSelectAll = useCallback(() => {
        if (selectedReports.size === filteredReports.length) {
            setSelectedReports(new Set());
        }
        else {
            setSelectedReports(new Set(filteredReports.map((r) => r.id)));
        }
    }, [selectedReports.size, filteredReports]);
    const handleBulkDelete = useCallback(() => {
        if (!onDeleteReport)
            return;
        const confirmed = window.confirm(`Are you sure you want to delete ${selectedReports.size} report(s)?`);
        if (confirmed) {
            selectedReports.forEach((reportId) => onDeleteReport(reportId));
            setSelectedReports(new Set());
        }
    }, [selectedReports, onDeleteReport]);
    const getLastExecution = useCallback((reportId) => {
        return executions
            .filter((e) => e.reportId === reportId)
            .sort((a, b) => new Date(b.startTime).getTime() - new Date(a.startTime).getTime())[0];
    }, [executions]);
    const renderReportCard = (report) => {
        const lastExecution = getLastExecution(report.id);
        const isSelected = selectedReports.has(report.id);
        return (React.createElement("div", { key: report.id, style: {
                ...styles.reportCard,
                ...(isSelected ? styles.reportCardSelected : {}),
            } },
            React.createElement("div", { style: styles.reportCardHeader },
                React.createElement("input", { type: "checkbox", checked: isSelected, onChange: () => toggleReportSelection(report.id), style: styles.checkbox }),
                React.createElement("span", { style: getStatusStyle(report.status) }, report.status)),
            React.createElement("div", { style: styles.reportCardContent },
                React.createElement("h3", { style: styles.reportName }, report.name),
                report.description && (React.createElement("p", { style: styles.reportDescription }, report.description)),
                React.createElement("div", { style: styles.reportMeta },
                    React.createElement("div", { style: styles.metaItem },
                        React.createElement("span", { style: styles.metaLabel }, "Type:"),
                        React.createElement("span", null, report.type)),
                    React.createElement("div", { style: styles.metaItem },
                        React.createElement("span", { style: styles.metaLabel }, "Version:"),
                        React.createElement("span", null,
                            "v",
                            report.version)),
                    report.metadata.category && (React.createElement("div", { style: styles.metaItem },
                        React.createElement("span", { style: styles.metaLabel }, "Category:"),
                        React.createElement("span", null, report.metadata.category)))),
                lastExecution && (React.createElement("div", { style: styles.lastExecution },
                    React.createElement("span", { style: styles.executionLabel }, "Last run:"),
                    React.createElement("span", { style: styles.executionTime }, new Date(lastExecution.startTime).toLocaleString()),
                    React.createElement("span", { style: getExecutionStatusStyle(lastExecution.status) }, lastExecution.status))),
                report.metadata.tags && report.metadata.tags.length > 0 && (React.createElement("div", { style: styles.tags },
                    report.metadata.tags.slice(0, 3).map((tag, index) => (React.createElement("span", { key: index, style: styles.tag }, tag))),
                    report.metadata.tags.length > 3 && (React.createElement("span", { style: styles.tagMore },
                        "+",
                        report.metadata.tags.length - 3))))),
            React.createElement("div", { style: styles.reportCardActions },
                onViewReport && (React.createElement("button", { onClick: () => onViewReport(report.id), style: styles.actionButton }, "\uD83D\uDC41\uFE0F View")),
                onEditReport && (React.createElement("button", { onClick: () => onEditReport(report), style: styles.actionButton }, "\u270E Edit")),
                onExecuteReport && (React.createElement("button", { onClick: () => onExecuteReport(report.id), style: styles.actionButton }, "\u25B6 Run")),
                React.createElement("div", { style: styles.moreMenu },
                    React.createElement("button", { style: styles.moreButton }, "\u22EE"),
                    React.createElement("div", { style: styles.moreMenuContent },
                        onDuplicateReport && (React.createElement("button", { onClick: () => onDuplicateReport(report) }, "Duplicate")),
                        onScheduleReport && (React.createElement("button", { onClick: () => onScheduleReport(report.id) }, "Schedule")),
                        onExportReport && (React.createElement("button", { onClick: () => onExportReport(report.id) }, "Export")),
                        onDeleteReport && (React.createElement("button", { onClick: () => onDeleteReport(report.id), style: { color: '#ef4444' } }, "Delete")))))));
    };
    const renderReportListItem = (report) => {
        const lastExecution = getLastExecution(report.id);
        const isSelected = selectedReports.has(report.id);
        return (React.createElement("div", { key: report.id, style: {
                ...styles.reportListItem,
                ...(isSelected ? styles.reportListItemSelected : {}),
            } },
            React.createElement("div", { style: styles.reportListLeft },
                React.createElement("input", { type: "checkbox", checked: isSelected, onChange: () => toggleReportSelection(report.id), style: styles.checkbox }),
                React.createElement("div", { style: styles.reportListInfo },
                    React.createElement("div", { style: styles.reportListName }, report.name),
                    React.createElement("div", { style: styles.reportListMeta },
                        React.createElement("span", { style: getStatusStyle(report.status) }, report.status),
                        React.createElement("span", null, "\u2022"),
                        React.createElement("span", null, report.type),
                        React.createElement("span", null, "\u2022"),
                        React.createElement("span", null,
                            "v",
                            report.version),
                        report.metadata.category && (React.createElement(React.Fragment, null,
                            React.createElement("span", null, "\u2022"),
                            React.createElement("span", null, report.metadata.category))),
                        React.createElement("span", null, "\u2022"),
                        React.createElement("span", null,
                            "Updated ",
                            new Date(report.metadata.updatedAt).toLocaleDateString())),
                    lastExecution && (React.createElement("div", { style: styles.reportListExecution },
                        "Last run: ",
                        new Date(lastExecution.startTime).toLocaleString(),
                        " -",
                        ' ',
                        React.createElement("span", { style: getExecutionStatusStyle(lastExecution.status) }, lastExecution.status))))),
            React.createElement("div", { style: styles.reportListActions },
                onViewReport && (React.createElement("button", { onClick: () => onViewReport(report.id), style: styles.actionButton }, "\uD83D\uDC41\uFE0F")),
                onEditReport && (React.createElement("button", { onClick: () => onEditReport(report), style: styles.actionButton }, "\u270E")),
                onExecuteReport && (React.createElement("button", { onClick: () => onExecuteReport(report.id), style: styles.actionButton }, "\u25B6")))));
    };
    return (React.createElement("div", { style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("h1", { style: styles.title }, "Reports"),
            React.createElement("div", { style: styles.headerActions },
                React.createElement("div", { style: styles.viewModeToggle },
                    React.createElement("button", { onClick: () => setViewMode('grid'), style: {
                            ...styles.viewModeButton,
                            ...(viewMode === 'grid' ? styles.viewModeButtonActive : {}),
                        } }, "\u229E"),
                    React.createElement("button", { onClick: () => setViewMode('list'), style: {
                            ...styles.viewModeButton,
                            ...(viewMode === 'list' ? styles.viewModeButtonActive : {}),
                        } }, "\u2630")),
                onCreateReport && (React.createElement("button", { onClick: onCreateReport, style: styles.createButton }, "+ New Report")))),
        React.createElement("div", { style: styles.filters },
            React.createElement("input", { type: "text", placeholder: "Search reports...", value: searchTerm, onChange: (e) => setSearchTerm(e.target.value), style: styles.searchInput }),
            React.createElement("select", { value: statusFilter, onChange: (e) => setStatusFilter(e.target.value), style: styles.filterSelect },
                React.createElement("option", { value: "all" }, "All Statuses"),
                React.createElement("option", { value: "draft" }, "Draft"),
                React.createElement("option", { value: "published" }, "Published"),
                React.createElement("option", { value: "archived" }, "Archived"),
                React.createElement("option", { value: "scheduled" }, "Scheduled")),
            React.createElement("select", { value: categoryFilter, onChange: (e) => setCategoryFilter(e.target.value), style: styles.filterSelect }, categories.map((category) => (React.createElement("option", { key: category, value: category }, category === 'all' ? 'All Categories' : category)))),
            React.createElement("select", { value: sortBy, onChange: (e) => setSortBy(e.target.value), style: styles.filterSelect },
                React.createElement("option", { value: "updated" }, "Recently Updated"),
                React.createElement("option", { value: "created" }, "Recently Created"),
                React.createElement("option", { value: "name" }, "Name (A-Z)"))),
        selectedReports.size > 0 && (React.createElement("div", { style: styles.bulkActions },
            React.createElement("div", { style: styles.bulkActionsLeft },
                React.createElement("input", { type: "checkbox", checked: selectedReports.size === filteredReports.length, onChange: toggleSelectAll, style: styles.checkbox }),
                React.createElement("span", { style: styles.selectedCount },
                    selectedReports.size,
                    " selected")),
            React.createElement("div", { style: styles.bulkActionsRight }, onDeleteReport && (React.createElement("button", { onClick: handleBulkDelete, style: styles.bulkDeleteButton }, "Delete Selected"))))),
        React.createElement("div", { style: styles.content }, filteredReports.length === 0 ? (React.createElement("div", { style: styles.emptyState },
            React.createElement("div", { style: styles.emptyStateIcon }, "\uD83D\uDCCA"),
            React.createElement("div", { style: styles.emptyStateText }, "No reports found"),
            React.createElement("div", { style: styles.emptyStateHint }, reports.length === 0
                ? 'Create your first report to get started'
                : 'Try adjusting your search or filter criteria'),
            onCreateReport && reports.length === 0 && (React.createElement("button", { onClick: onCreateReport, style: styles.emptyStateButton }, "+ Create Report")))) : viewMode === 'grid' ? (React.createElement("div", { style: styles.reportsGrid }, filteredReports.map(renderReportCard))) : (React.createElement("div", { style: styles.reportsList }, filteredReports.map(renderReportListItem)))),
        React.createElement("div", { style: styles.footer },
            React.createElement("div", { style: styles.stats },
                React.createElement("div", { style: styles.statItem },
                    React.createElement("span", { style: styles.statValue }, reports.length),
                    React.createElement("span", { style: styles.statLabel }, "Total Reports")),
                React.createElement("div", { style: styles.statItem },
                    React.createElement("span", { style: styles.statValue }, reports.filter((r) => r.status === 'published').length),
                    React.createElement("span", { style: styles.statLabel }, "Published")),
                React.createElement("div", { style: styles.statItem },
                    React.createElement("span", { style: styles.statValue }, reports.filter((r) => r.schedule?.enabled).length),
                    React.createElement("span", { style: styles.statLabel }, "Scheduled")),
                React.createElement("div", { style: styles.statItem },
                    React.createElement("span", { style: styles.statValue }, executions.length),
                    React.createElement("span", { style: styles.statLabel }, "Total Executions"))))));
};
function getStatusStyle(status) {
    const baseStyle = {
        fontSize: '11px',
        padding: '2px 8px',
        borderRadius: '12px',
        fontWeight: 600,
    };
    const colors = {
        draft: { bg: '#f1f5f9', color: '#475569' },
        published: { bg: '#d1fae5', color: '#065f46' },
        archived: { bg: '#fef3c7', color: '#92400e' },
        scheduled: { bg: '#dbeafe', color: '#1e40af' },
        running: { bg: '#e0e7ff', color: '#3730a3' },
        completed: { bg: '#d1fae5', color: '#065f46' },
        failed: { bg: '#fee2e2', color: '#991b1b' },
    };
    return {
        ...baseStyle,
        backgroundColor: colors[status]?.bg || '#f1f5f9',
        color: colors[status]?.color || '#475569',
    };
}
function getExecutionStatusStyle(status) {
    const colors = {
        completed: '#10b981',
        running: '#3b82f6',
        failed: '#ef4444',
        pending: '#64748b',
        cancelled: '#94a3b8',
    };
    return {
        color: colors[status] || '#64748b',
        fontWeight: 600,
    };
}
const styles = {
    container: {
        display: 'flex',
        flexDirection: 'column',
        height: '100vh',
        backgroundColor: '#f8fafc',
        fontFamily: 'Inter, system-ui, sans-serif',
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '20px 24px',
        backgroundColor: '#ffffff',
        borderBottom: '1px solid #e2e8f0',
    },
    title: {
        fontSize: '24px',
        fontWeight: 700,
        margin: 0,
        color: '#1e293b',
    },
    headerActions: {
        display: 'flex',
        gap: '12px',
    },
    viewModeToggle: {
        display: 'flex',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        overflow: 'hidden',
    },
    viewModeButton: {
        padding: '8px 12px',
        border: 'none',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '16px',
    },
    viewModeButtonActive: {
        backgroundColor: '#2563eb',
        color: '#ffffff',
    },
    createButton: {
        padding: '8px 16px',
        border: 'none',
        borderRadius: '6px',
        backgroundColor: '#2563eb',
        color: '#ffffff',
        fontSize: '14px',
        fontWeight: 600,
        cursor: 'pointer',
    },
    filters: {
        display: 'flex',
        gap: '12px',
        padding: '16px 24px',
        backgroundColor: '#ffffff',
        borderBottom: '1px solid #e2e8f0',
    },
    searchInput: {
        flex: 1,
        padding: '8px 12px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        fontSize: '14px',
    },
    filterSelect: {
        padding: '8px 12px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        fontSize: '14px',
        cursor: 'pointer',
    },
    bulkActions: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '12px 24px',
        backgroundColor: '#eff6ff',
        borderBottom: '1px solid #dbeafe',
    },
    bulkActionsLeft: {
        display: 'flex',
        alignItems: 'center',
        gap: '12px',
    },
    bulkActionsRight: {
        display: 'flex',
        gap: '8px',
    },
    selectedCount: {
        fontSize: '14px',
        fontWeight: 600,
        color: '#1e40af',
    },
    bulkDeleteButton: {
        padding: '6px 12px',
        border: '1px solid #fecaca',
        borderRadius: '4px',
        backgroundColor: '#fef2f2',
        color: '#dc2626',
        fontSize: '13px',
        fontWeight: 500,
        cursor: 'pointer',
    },
    content: {
        flex: 1,
        overflow: 'auto',
        padding: '24px',
    },
    reportsGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(350px, 1fr))',
        gap: '20px',
    },
    reportCard: {
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        padding: '16px',
        transition: 'all 0.2s',
    },
    reportCardSelected: {
        borderColor: '#2563eb',
        boxShadow: '0 0 0 3px rgba(37, 99, 235, 0.1)',
    },
    reportCardHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '12px',
    },
    checkbox: {
        width: '16px',
        height: '16px',
        cursor: 'pointer',
    },
    reportCardContent: {
        marginBottom: '16px',
    },
    reportName: {
        fontSize: '16px',
        fontWeight: 600,
        margin: '0 0 8px 0',
        color: '#1e293b',
    },
    reportDescription: {
        fontSize: '13px',
        color: '#64748b',
        margin: '0 0 12px 0',
        lineHeight: 1.5,
    },
    reportMeta: {
        display: 'flex',
        flexWrap: 'wrap',
        gap: '8px 16px',
        marginBottom: '12px',
    },
    metaItem: {
        fontSize: '12px',
        display: 'flex',
        gap: '4px',
    },
    metaLabel: {
        color: '#94a3b8',
    },
    lastExecution: {
        fontSize: '11px',
        color: '#64748b',
        padding: '8px',
        backgroundColor: '#f8fafc',
        borderRadius: '4px',
        marginBottom: '8px',
    },
    executionLabel: {
        fontWeight: 500,
    },
    executionTime: {
        marginLeft: '4px',
    },
    tags: {
        display: 'flex',
        flexWrap: 'wrap',
        gap: '4px',
    },
    tag: {
        fontSize: '10px',
        padding: '2px 6px',
        backgroundColor: '#f1f5f9',
        color: '#475569',
        borderRadius: '4px',
    },
    tagMore: {
        fontSize: '10px',
        padding: '2px 6px',
        color: '#64748b',
    },
    reportCardActions: {
        display: 'flex',
        gap: '8px',
        paddingTop: '12px',
        borderTop: '1px solid #e2e8f0',
    },
    actionButton: {
        flex: 1,
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '12px',
        fontWeight: 500,
    },
    moreMenu: {
        position: 'relative',
    },
    moreButton: {
        padding: '6px 12px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '16px',
    },
    moreMenuContent: {
        display: 'none',
        position: 'absolute',
        right: 0,
        top: '100%',
        marginTop: '4px',
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
        zIndex: 10,
    },
    reportsList: {
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
    },
    reportListItem: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '16px',
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        transition: 'all 0.2s',
    },
    reportListItemSelected: {
        borderColor: '#2563eb',
        boxShadow: '0 0 0 3px rgba(37, 99, 235, 0.1)',
    },
    reportListLeft: {
        display: 'flex',
        gap: '16px',
        flex: 1,
        alignItems: 'center',
    },
    reportListInfo: {
        flex: 1,
    },
    reportListName: {
        fontSize: '15px',
        fontWeight: 600,
        marginBottom: '4px',
        color: '#1e293b',
    },
    reportListMeta: {
        fontSize: '12px',
        color: '#64748b',
        display: 'flex',
        gap: '8px',
        flexWrap: 'wrap',
        alignItems: 'center',
    },
    reportListExecution: {
        fontSize: '11px',
        color: '#94a3b8',
        marginTop: '4px',
    },
    reportListActions: {
        display: 'flex',
        gap: '8px',
    },
    emptyState: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '64px',
        gap: '16px',
    },
    emptyStateIcon: {
        fontSize: '64px',
    },
    emptyStateText: {
        fontSize: '18px',
        color: '#64748b',
        fontWeight: 600,
    },
    emptyStateHint: {
        fontSize: '14px',
        color: '#94a3b8',
    },
    emptyStateButton: {
        padding: '12px 24px',
        border: 'none',
        borderRadius: '6px',
        backgroundColor: '#2563eb',
        color: '#ffffff',
        fontSize: '14px',
        fontWeight: 600,
        cursor: 'pointer',
        marginTop: '8px',
    },
    footer: {
        padding: '16px 24px',
        backgroundColor: '#ffffff',
        borderTop: '1px solid #e2e8f0',
    },
    stats: {
        display: 'flex',
        justifyContent: 'space-around',
    },
    statItem: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: '4px',
    },
    statValue: {
        fontSize: '24px',
        fontWeight: 700,
        color: '#1e293b',
    },
    statLabel: {
        fontSize: '12px',
        color: '#64748b',
        textTransform: 'uppercase',
        letterSpacing: '0.5px',
    },
};
export default ReportDashboard;
//# sourceMappingURL=ReportDashboard.js.map