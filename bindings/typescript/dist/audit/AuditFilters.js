import React, { useState, useEffect } from 'react';
export const AuditFilters = ({ filters, onChange, onReset, }) => {
    const [isExpanded, setIsExpanded] = useState(false);
    const [localFilters, setLocalFilters] = useState(filters);
    useEffect(() => {
        setLocalFilters(filters);
    }, [filters]);
    const handleApplyFilters = () => {
        onChange(localFilters);
    };
    const handleResetFilters = () => {
        setLocalFilters({});
        onReset();
    };
    const updateFilter = (key, value) => {
        setLocalFilters((prev) => ({
            ...prev,
            [key]: value,
        }));
    };
    const toggleEventType = (eventType) => {
        const current = localFilters.event_types || [];
        const updated = current.includes(eventType)
            ? current.filter((t) => t !== eventType)
            : [...current, eventType];
        updateFilter('event_types', updated.length > 0 ? updated : undefined);
    };
    const toggleSeverity = (severity) => {
        const current = localFilters.severities || [];
        const updated = current.includes(severity)
            ? current.filter((s) => s !== severity)
            : [...current, severity];
        updateFilter('severities', updated.length > 0 ? updated : undefined);
    };
    const toggleStatus = (status) => {
        const current = localFilters.statuses || [];
        const updated = current.includes(status)
            ? current.filter((s) => s !== status)
            : [...current, status];
        updateFilter('statuses', updated.length > 0 ? updated : undefined);
    };
    const toggleResourceType = (resourceType) => {
        const current = localFilters.resource_types || [];
        const updated = current.includes(resourceType)
            ? current.filter((r) => r !== resourceType)
            : [...current, resourceType];
        updateFilter('resource_types', updated.length > 0 ? updated : undefined);
    };
    const activeFilterCount = Object.values(localFilters).filter((v) => v !== undefined && (Array.isArray(v) ? v.length > 0 : true)).length;
    return (React.createElement("div", { className: "audit-filters" },
        React.createElement("div", { className: "quick-filters" },
            React.createElement("div", { className: "search-box" },
                React.createElement("input", { type: "text", placeholder: "Search events, users, resources...", value: localFilters.search_query || '', onChange: (e) => updateFilter('search_query', e.target.value || undefined), className: "search-input" }),
                React.createElement("span", { className: "search-icon" }, "\uD83D\uDD0D")),
            React.createElement("div", { className: "quick-filter-buttons" },
                React.createElement("button", { className: `filter-chip ${localFilters.anomaly_only ? 'active' : ''}`, onClick: () => updateFilter('anomaly_only', !localFilters.anomaly_only) }, "Anomalies Only"),
                React.createElement("button", { className: `filter-chip ${localFilters.min_risk_score !== undefined ? 'active' : ''}`, onClick: () => updateFilter('min_risk_score', localFilters.min_risk_score !== undefined ? undefined : 70) }, "High Risk"),
                React.createElement("button", { className: "filter-toggle", onClick: () => setIsExpanded(!isExpanded) },
                    "Advanced Filters",
                    activeFilterCount > 0 && (React.createElement("span", { className: "filter-count" }, activeFilterCount)),
                    React.createElement("span", { className: `toggle-icon ${isExpanded ? 'expanded' : ''}` }, "\u25BC"))),
            React.createElement("div", { className: "filter-actions" },
                React.createElement("button", { className: "btn btn-secondary", onClick: handleResetFilters }, "Reset"),
                React.createElement("button", { className: "btn btn-primary", onClick: handleApplyFilters }, "Apply Filters"))),
        isExpanded && (React.createElement("div", { className: "advanced-filters" },
            React.createElement("div", { className: "filter-section" },
                React.createElement("h4", null, "Date Range"),
                React.createElement("div", { className: "date-range" },
                    React.createElement("div", { className: "date-input-group" },
                        React.createElement("label", null, "Start Date"),
                        React.createElement("input", { type: "datetime-local", value: localFilters.start_date
                                ? formatDateForInput(localFilters.start_date)
                                : '', onChange: (e) => updateFilter('start_date', e.target.value ? new Date(e.target.value) : undefined) })),
                    React.createElement("div", { className: "date-input-group" },
                        React.createElement("label", null, "End Date"),
                        React.createElement("input", { type: "datetime-local", value: localFilters.end_date
                                ? formatDateForInput(localFilters.end_date)
                                : '', onChange: (e) => updateFilter('end_date', e.target.value ? new Date(e.target.value) : undefined) })))),
            React.createElement("div", { className: "filter-section" },
                React.createElement("h4", null, "Event Types"),
                React.createElement("div", { className: "filter-chips" }, EVENT_TYPE_GROUPS.map((group) => (React.createElement("div", { key: group.category, className: "filter-group" },
                    React.createElement("h5", null, group.category),
                    React.createElement("div", { className: "chip-grid" }, group.types.map((type) => (React.createElement("button", { key: type, className: `filter-chip ${localFilters.event_types?.includes(type) ? 'active' : ''}`, onClick: () => toggleEventType(type) }, formatEventType(type)))))))))),
            React.createElement("div", { className: "filter-section" },
                React.createElement("h4", null, "Severity"),
                React.createElement("div", { className: "severity-filters" }, ['low', 'medium', 'high', 'critical'].map((severity) => (React.createElement("button", { key: severity, className: `severity-chip severity-${severity} ${localFilters.severities?.includes(severity) ? 'active' : ''}`, onClick: () => toggleSeverity(severity) }, severity.toUpperCase()))))),
            React.createElement("div", { className: "filter-section" },
                React.createElement("h4", null, "Status"),
                React.createElement("div", { className: "status-filters" }, ['success', 'failure', 'pending', 'blocked'].map((status) => (React.createElement("button", { key: status, className: `status-chip status-${status} ${localFilters.statuses?.includes(status) ? 'active' : ''}`, onClick: () => toggleStatus(status) }, status.charAt(0).toUpperCase() + status.slice(1)))))),
            React.createElement("div", { className: "filter-section" },
                React.createElement("h4", null, "Resource Types"),
                React.createElement("div", { className: "resource-filters" }, RESOURCE_TYPES.map((resource) => (React.createElement("button", { key: resource.value, className: `filter-chip ${localFilters.resource_types?.includes(resource.value)
                        ? 'active'
                        : ''}`, onClick: () => toggleResourceType(resource.value) }, resource.label))))),
            React.createElement("div", { className: "filter-section" },
                React.createElement("h4", null, "User & Session"),
                React.createElement("div", { className: "input-grid" },
                    React.createElement("div", { className: "input-group" },
                        React.createElement("label", null, "User ID or Email"),
                        React.createElement("input", { type: "text", placeholder: "Enter user ID or email", value: localFilters.user_ids?.join(', ') || '', onChange: (e) => updateFilter('user_ids', e.target.value
                                ? e.target.value.split(',').map((s) => s.trim())
                                : undefined) })),
                    React.createElement("div", { className: "input-group" },
                        React.createElement("label", null, "IP Address"),
                        React.createElement("input", { type: "text", placeholder: "Enter IP address", value: localFilters.ip_address || '', onChange: (e) => updateFilter('ip_address', e.target.value || undefined) })),
                    React.createElement("div", { className: "input-group" },
                        React.createElement("label", null, "Session ID"),
                        React.createElement("input", { type: "text", placeholder: "Enter session ID", value: localFilters.session_id || '', onChange: (e) => updateFilter('session_id', e.target.value || undefined) })))),
            React.createElement("div", { className: "filter-section" },
                React.createElement("h4", null, "Risk Score"),
                React.createElement("div", { className: "risk-score-filter" },
                    React.createElement("label", null,
                        "Minimum Risk Score: ",
                        localFilters.min_risk_score || 0),
                    React.createElement("input", { type: "range", min: "0", max: "100", step: "10", value: localFilters.min_risk_score || 0, onChange: (e) => updateFilter('min_risk_score', parseInt(e.target.value) || undefined), className: "risk-slider" }),
                    React.createElement("div", { className: "risk-scale" },
                        React.createElement("span", null, "0"),
                        React.createElement("span", null, "25"),
                        React.createElement("span", null, "50"),
                        React.createElement("span", null, "75"),
                        React.createElement("span", null, "100"))))))));
};
const EVENT_TYPE_GROUPS = [
    {
        category: 'User Events',
        types: [
            'user.login',
            'user.logout',
            'user.created',
            'user.updated',
            'user.deleted',
            'user.password_changed',
            'user.mfa_enabled',
            'user.mfa_disabled',
        ],
    },
    {
        category: 'Role & Permission Events',
        types: [
            'role.created',
            'role.updated',
            'role.deleted',
            'role.assigned',
            'role.revoked',
            'permission.granted',
            'permission.revoked',
        ],
    },
    {
        category: 'Resource Events',
        types: [
            'resource.created',
            'resource.read',
            'resource.updated',
            'resource.deleted',
            'resource.shared',
            'resource.exported',
            'resource.imported',
        ],
    },
    {
        category: 'Data Events',
        types: [
            'data.accessed',
            'data.modified',
            'data.deleted',
            'data.exported',
        ],
    },
    {
        category: 'Security Events',
        types: [
            'security.breach_attempt',
            'security.unauthorized_access',
            'security.suspicious_activity',
        ],
    },
    {
        category: 'System Events',
        types: [
            'system.started',
            'system.stopped',
            'system.error',
            'config.changed',
        ],
    },
    {
        category: 'Compliance Events',
        types: [
            'compliance.violation',
            'audit.tamper_attempt',
        ],
    },
];
const RESOURCE_TYPES = [
    { value: 'project', label: 'Projects' },
    { value: 'drawing', label: 'Drawings' },
    { value: 'model', label: '3D Models' },
    { value: 'layer', label: 'Layers' },
    { value: 'template', label: 'Templates' },
    { value: 'user', label: 'Users' },
    { value: 'role', label: 'Roles' },
    { value: 'team', label: 'Teams' },
    { value: 'organization', label: 'Organizations' },
    { value: 'settings', label: 'Settings' },
    { value: 'audit_log', label: 'Audit Logs' },
    { value: 'report', label: 'Reports' },
    { value: 'plugin', label: 'Plugins' },
    { value: 'workflow', label: 'Workflows' },
];
function formatEventType(eventType) {
    const parts = eventType.split('.');
    return parts[parts.length - 1]
        .split('_')
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ');
}
function formatDateForInput(date) {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    return `${year}-${month}-${day}T${hours}:${minutes}`;
}
//# sourceMappingURL=AuditFilters.js.map