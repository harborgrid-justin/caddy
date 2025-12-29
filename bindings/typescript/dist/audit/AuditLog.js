import React, { useState, useEffect, useCallback } from 'react';
import { AuditFilters } from './AuditFilters';
export const AuditLog = ({ organizationId, defaultFilters = {}, onEventSelect, }) => {
    const [events, setEvents] = useState([]);
    const [totalCount, setTotalCount] = useState(0);
    const [loading, setLoading] = useState(false);
    const [filters, setFilters] = useState(defaultFilters);
    const [page, setPage] = useState(1);
    const [pageSize, setPageSize] = useState(50);
    const [selectedEvents, setSelectedEvents] = useState(new Set());
    const [sortField, setSortField] = useState('timestamp');
    const [sortDirection, setSortDirection] = useState('desc');
    useEffect(() => {
        loadEvents();
    }, [filters, page, pageSize, sortField, sortDirection, organizationId]);
    const loadEvents = async () => {
        setLoading(true);
        try {
            const params = new URLSearchParams({
                page: page.toString(),
                page_size: pageSize.toString(),
                sort_field: sortField,
                sort_direction: sortDirection,
                ...(organizationId && { organization_id: organizationId }),
            });
            if (filters.event_types?.length) {
                params.append('event_types', filters.event_types.join(','));
            }
            if (filters.severities?.length) {
                params.append('severities', filters.severities.join(','));
            }
            if (filters.statuses?.length) {
                params.append('statuses', filters.statuses.join(','));
            }
            if (filters.user_ids?.length) {
                params.append('user_ids', filters.user_ids.join(','));
            }
            if (filters.resource_types?.length) {
                params.append('resource_types', filters.resource_types.join(','));
            }
            if (filters.start_date) {
                params.append('start_date', filters.start_date.toISOString());
            }
            if (filters.end_date) {
                params.append('end_date', filters.end_date.toISOString());
            }
            if (filters.search_query) {
                params.append('search', filters.search_query);
            }
            if (filters.anomaly_only) {
                params.append('anomaly_only', 'true');
            }
            if (filters.min_risk_score !== undefined) {
                params.append('min_risk_score', filters.min_risk_score.toString());
            }
            const response = await fetch(`/api/audit/events?${params}`);
            const data = await response.json();
            setEvents(data.events || []);
            setTotalCount(data.total || 0);
        }
        catch (error) {
            console.error('Failed to load audit events:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const handleFilterChange = useCallback((newFilters) => {
        setFilters(newFilters);
        setPage(1);
    }, []);
    const handleSort = (field) => {
        if (field === sortField) {
            setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
        }
        else {
            setSortField(field);
            setSortDirection('desc');
        }
    };
    const handleSelectEvent = (eventId) => {
        const newSelected = new Set(selectedEvents);
        if (newSelected.has(eventId)) {
            newSelected.delete(eventId);
        }
        else {
            newSelected.add(eventId);
        }
        setSelectedEvents(newSelected);
    };
    const handleSelectAll = () => {
        if (selectedEvents.size === events.length) {
            setSelectedEvents(new Set());
        }
        else {
            setSelectedEvents(new Set(events.map((e) => e.id)));
        }
    };
    const handleBulkExport = async () => {
        const selectedEventsList = events.filter((e) => selectedEvents.has(e.id));
        const csvContent = generateCSV(selectedEventsList);
        downloadFile(csvContent, 'audit-logs.csv', 'text/csv');
    };
    const totalPages = Math.ceil(totalCount / pageSize);
    return (React.createElement("div", { className: "audit-log" },
        React.createElement("div", { className: "audit-log-header" },
            React.createElement("div", null,
                React.createElement("h2", null, "Audit Logs"),
                React.createElement("p", { className: "subtitle" },
                    totalCount.toLocaleString(),
                    " events found")),
            React.createElement("div", { className: "header-actions" },
                React.createElement("button", { className: "btn btn-secondary", onClick: handleBulkExport, disabled: selectedEvents.size === 0 },
                    "Export Selected (",
                    selectedEvents.size,
                    ")"),
                React.createElement("button", { className: "btn btn-primary", onClick: loadEvents }, "Refresh"))),
        React.createElement(AuditFilters, { filters: filters, onChange: handleFilterChange, onReset: () => handleFilterChange({}) }),
        React.createElement("div", { className: "table-controls" },
            React.createElement("div", { className: "page-size-selector" },
                React.createElement("label", null, "Show:"),
                React.createElement("select", { value: pageSize, onChange: (e) => {
                        setPageSize(Number(e.target.value));
                        setPage(1);
                    } },
                    React.createElement("option", { value: 25 }, "25"),
                    React.createElement("option", { value: 50 }, "50"),
                    React.createElement("option", { value: 100 }, "100"),
                    React.createElement("option", { value: 250 }, "250")),
                React.createElement("span", null, "per page")),
            React.createElement("div", { className: "view-options" },
                React.createElement("button", { className: `view-btn ${selectedEvents.size === 0 ? 'active' : ''}`, onClick: () => setSelectedEvents(new Set()) }, "All Events"),
                React.createElement("button", { className: `view-btn ${filters.anomaly_only ? 'active' : ''}`, onClick: () => handleFilterChange({ ...filters, anomaly_only: !filters.anomaly_only }) }, "Anomalies Only"))),
        React.createElement("div", { className: "table-container" }, loading ? (React.createElement("div", { className: "loading-state" },
            React.createElement("div", { className: "loading-spinner" }),
            React.createElement("p", null, "Loading audit events..."))) : events.length === 0 ? (React.createElement("div", { className: "empty-state" },
            React.createElement("p", null, "No audit events found matching your filters"),
            React.createElement("button", { onClick: () => handleFilterChange({}) }, "Clear Filters"))) : (React.createElement("table", { className: "audit-table" },
            React.createElement("thead", null,
                React.createElement("tr", null,
                    React.createElement("th", { className: "checkbox-cell" },
                        React.createElement("input", { type: "checkbox", checked: selectedEvents.size === events.length, onChange: handleSelectAll })),
                    React.createElement("th", { onClick: () => handleSort('timestamp'), className: "sortable" },
                        "Timestamp",
                        sortField === 'timestamp' && (React.createElement(SortIndicator, { direction: sortDirection }))),
                    React.createElement("th", { onClick: () => handleSort('event_type'), className: "sortable" },
                        "Event Type",
                        sortField === 'event_type' && (React.createElement(SortIndicator, { direction: sortDirection }))),
                    React.createElement("th", { onClick: () => handleSort('user_email'), className: "sortable" },
                        "User",
                        sortField === 'user_email' && (React.createElement(SortIndicator, { direction: sortDirection }))),
                    React.createElement("th", null, "Resource"),
                    React.createElement("th", null, "Action"),
                    React.createElement("th", { onClick: () => handleSort('status'), className: "sortable" },
                        "Status",
                        sortField === 'status' && (React.createElement(SortIndicator, { direction: sortDirection }))),
                    React.createElement("th", { onClick: () => handleSort('severity'), className: "sortable" },
                        "Severity",
                        sortField === 'severity' && (React.createElement(SortIndicator, { direction: sortDirection }))),
                    React.createElement("th", null, "Risk"),
                    React.createElement("th", null, "Actions"))),
            React.createElement("tbody", null, events.map((event) => (React.createElement("tr", { key: event.id, className: `${event.anomaly_detected ? 'anomaly-row' : ''} ${selectedEvents.has(event.id) ? 'selected' : ''}` },
                React.createElement("td", { className: "checkbox-cell" },
                    React.createElement("input", { type: "checkbox", checked: selectedEvents.has(event.id), onChange: () => handleSelectEvent(event.id) })),
                React.createElement("td", { className: "timestamp-cell" }, formatTimestamp(event.timestamp)),
                React.createElement("td", { className: "event-type-cell" },
                    React.createElement("div", { className: "event-type-wrapper" },
                        formatEventType(event.event_type),
                        event.anomaly_detected && (React.createElement("span", { className: "anomaly-badge", title: "Anomaly detected" }, "!")))),
                React.createElement("td", null,
                    React.createElement("div", { className: "user-info" },
                        React.createElement("div", { className: "user-email" }, event.user_email || 'System'),
                        React.createElement("div", { className: "user-ip" }, event.user_ip_address))),
                React.createElement("td", null, event.resource_name || event.resource_id ? (React.createElement("div", { className: "resource-info" },
                    React.createElement("div", { className: "resource-name" }, event.resource_name || event.resource_id),
                    event.resource_type && (React.createElement("div", { className: "resource-type" }, event.resource_type)))) : ('-')),
                React.createElement("td", null, event.action),
                React.createElement("td", null,
                    React.createElement(StatusBadge, { status: event.status })),
                React.createElement("td", null,
                    React.createElement(SeverityBadge, { severity: event.severity })),
                React.createElement("td", null, event.risk_score !== undefined && (React.createElement(RiskScoreBadge, { score: event.risk_score }))),
                React.createElement("td", { className: "actions-cell" },
                    React.createElement("button", { className: "btn-icon", onClick: () => onEventSelect?.(event), title: "View Details" }, "\uD83D\uDC41\uFE0F"))))))))),
        totalPages > 1 && (React.createElement("div", { className: "pagination" },
            React.createElement("button", { className: "pagination-btn", onClick: () => setPage(1), disabled: page === 1 }, "First"),
            React.createElement("button", { className: "pagination-btn", onClick: () => setPage(page - 1), disabled: page === 1 }, "Previous"),
            React.createElement("div", { className: "page-numbers" }, Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                const pageNum = Math.max(1, Math.min(page - 2 + i, totalPages - 4 + i));
                return (React.createElement("button", { key: pageNum, className: `page-number ${page === pageNum ? 'active' : ''}`, onClick: () => setPage(pageNum) }, pageNum));
            })),
            React.createElement("button", { className: "pagination-btn", onClick: () => setPage(page + 1), disabled: page === totalPages }, "Next"),
            React.createElement("button", { className: "pagination-btn", onClick: () => setPage(totalPages), disabled: page === totalPages }, "Last"),
            React.createElement("div", { className: "page-info" },
                "Page ",
                page,
                " of ",
                totalPages)))));
};
function SortIndicator({ direction }) {
    return React.createElement("span", { className: "sort-indicator" }, direction === 'asc' ? '▲' : '▼');
}
function StatusBadge({ status }) {
    const colors = {
        success: 'green',
        failure: 'red',
        pending: 'yellow',
        blocked: 'gray',
    };
    return (React.createElement("span", { className: `badge badge-${colors[status]}` }, status));
}
function SeverityBadge({ severity }) {
    const colors = {
        low: 'green',
        medium: 'yellow',
        high: 'orange',
        critical: 'red',
    };
    return (React.createElement("span", { className: `badge badge-${colors[severity]}` }, severity.toUpperCase()));
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
    return (React.createElement("div", { className: "risk-score" },
        React.createElement("span", { className: `risk-badge badge-${getColor(score)}` }, score.toFixed(0))));
}
function formatTimestamp(timestamp) {
    const date = new Date(timestamp);
    return new Intl.DateTimeFormat('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
    }).format(date);
}
function formatEventType(eventType) {
    return eventType
        .split('.')
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join(' ');
}
function generateCSV(events) {
    const headers = [
        'Timestamp',
        'Event Type',
        'User Email',
        'User IP',
        'Resource Type',
        'Resource ID',
        'Action',
        'Status',
        'Severity',
        'Risk Score',
        'Description',
    ];
    const rows = events.map((event) => [
        event.timestamp,
        event.event_type,
        event.user_email || '',
        event.user_ip_address,
        event.resource_type || '',
        event.resource_id || '',
        event.action,
        event.status,
        event.severity,
        event.risk_score?.toString() || '',
        event.description,
    ]);
    return [
        headers.join(','),
        ...rows.map((row) => row.map((cell) => `"${cell.toString().replace(/"/g, '""')}"`).join(',')),
    ].join('\n');
}
function downloadFile(content, filename, mimeType) {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
}
//# sourceMappingURL=AuditLog.js.map