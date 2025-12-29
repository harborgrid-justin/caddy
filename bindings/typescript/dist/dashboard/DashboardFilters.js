import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { useDashboard } from './DashboardLayout';
export const DashboardFiltersComponent = ({ filters, onChange, departments = [], regions = [], users = [], statuses = [], showDateRange = true, showDepartments = true, showRegions = true, showUsers = true, showStatuses = true, enableSavedFilters = true, className = '', }) => {
    const [localFilters, setLocalFilters] = useState(filters);
    const [isExpanded, setIsExpanded] = useState(false);
    const [savedFilters, setSavedFilters] = useState([]);
    const [filterName, setFilterName] = useState('');
    const { theme, accessibility } = useDashboard();
    const timeRangePresets = [
        { value: '1h', label: 'Last Hour' },
        { value: '24h', label: 'Last 24 Hours' },
        { value: '7d', label: 'Last 7 Days' },
        { value: '30d', label: 'Last 30 Days' },
        { value: '90d', label: 'Last 90 Days' },
        { value: '1y', label: 'Last Year' },
        { value: 'custom', label: 'Custom Range' },
    ];
    useEffect(() => {
        if (enableSavedFilters) {
            const saved = localStorage.getItem('dashboard-saved-filters');
            if (saved) {
                try {
                    setSavedFilters(JSON.parse(saved));
                }
                catch (error) {
                    console.error('Failed to load saved filters:', error);
                }
            }
        }
    }, [enableSavedFilters]);
    useEffect(() => {
        setLocalFilters(filters);
    }, [filters]);
    const handleFilterChange = useCallback((updates) => {
        const updated = { ...localFilters, ...updates };
        setLocalFilters(updated);
        onChange(updated);
    }, [localFilters, onChange]);
    const handleTimeRangeChange = useCallback((timeRange) => {
        handleFilterChange({
            timeRange,
            ...(timeRange !== 'custom' && { startDate: undefined, endDate: undefined }),
        });
    }, [handleFilterChange]);
    const handleCustomDateRange = useCallback((startDate, endDate) => {
        handleFilterChange({
            timeRange: 'custom',
            startDate,
            endDate,
        });
    }, [handleFilterChange]);
    const toggleDepartment = useCallback((department) => {
        const current = localFilters.departments || [];
        const updated = current.includes(department)
            ? current.filter((d) => d !== department)
            : [...current, department];
        handleFilterChange({ departments: updated });
    }, [localFilters.departments, handleFilterChange]);
    const toggleRegion = useCallback((region) => {
        const current = localFilters.regions || [];
        const updated = current.includes(region)
            ? current.filter((r) => r !== region)
            : [...current, region];
        handleFilterChange({ regions: updated });
    }, [localFilters.regions, handleFilterChange]);
    const toggleUser = useCallback((user) => {
        const current = localFilters.users || [];
        const updated = current.includes(user)
            ? current.filter((u) => u !== user)
            : [...current, user];
        handleFilterChange({ users: updated });
    }, [localFilters.users, handleFilterChange]);
    const toggleStatus = useCallback((status) => {
        const current = localFilters.statuses || [];
        const updated = current.includes(status)
            ? current.filter((s) => s !== status)
            : [...current, status];
        handleFilterChange({ statuses: updated });
    }, [localFilters.statuses, handleFilterChange]);
    const resetFilters = useCallback(() => {
        const reset = {
            timeRange: '24h',
            departments: undefined,
            regions: undefined,
            users: undefined,
            statuses: undefined,
            custom: undefined,
        };
        setLocalFilters(reset);
        onChange(reset);
    }, [onChange]);
    const saveFilters = useCallback(() => {
        if (!filterName) {
            alert('Please enter a name for this filter');
            return;
        }
        const newFilter = {
            id: Date.now().toString(),
            name: filterName,
            filters: localFilters,
            createdAt: new Date().toISOString(),
        };
        const updated = [...savedFilters, newFilter];
        setSavedFilters(updated);
        localStorage.setItem('dashboard-saved-filters', JSON.stringify(updated));
        setFilterName('');
    }, [filterName, localFilters, savedFilters]);
    const loadSavedFilter = useCallback((savedFilter) => {
        setLocalFilters(savedFilter.filters);
        onChange(savedFilter.filters);
    }, [onChange]);
    const deleteSavedFilter = useCallback((filterId) => {
        const updated = savedFilters.filter((f) => f.id !== filterId);
        setSavedFilters(updated);
        localStorage.setItem('dashboard-saved-filters', JSON.stringify(updated));
    }, [savedFilters]);
    const activeFilterCount = useMemo(() => {
        let count = 0;
        if (localFilters.departments?.length)
            count++;
        if (localFilters.regions?.length)
            count++;
        if (localFilters.users?.length)
            count++;
        if (localFilters.statuses?.length)
            count++;
        if (localFilters.custom && Object.keys(localFilters.custom).length)
            count++;
        return count;
    }, [localFilters]);
    return (React.createElement("div", { className: `dashboard-filters ${className}`, style: styles.container, role: "region", "aria-label": "Dashboard filters" },
        React.createElement("div", { style: styles.header },
            React.createElement("h3", { style: styles.title },
                "Filters",
                activeFilterCount > 0 && (React.createElement("span", { style: styles.activeCount, "aria-label": `${activeFilterCount} active filters` }, activeFilterCount))),
            React.createElement("div", { style: styles.headerActions },
                React.createElement("button", { onClick: () => setIsExpanded(!isExpanded), style: styles.expandButton, "aria-label": isExpanded ? 'Collapse filters' : 'Expand filters', "aria-expanded": isExpanded }, isExpanded ? '▲' : '▼'),
                activeFilterCount > 0 && (React.createElement("button", { onClick: resetFilters, style: styles.resetButton, "aria-label": "Reset all filters" }, "Reset")))),
        isExpanded && (React.createElement("div", { style: styles.content },
            showDateRange && (React.createElement("div", { style: styles.filterGroup },
                React.createElement("label", { style: styles.label, id: "time-range-label" }, "Time Range"),
                React.createElement("select", { value: localFilters.timeRange, onChange: (e) => handleTimeRangeChange(e.target.value), style: styles.select, "aria-labelledby": "time-range-label" }, timeRangePresets.map((preset) => (React.createElement("option", { key: preset.value, value: preset.value }, preset.label)))),
                localFilters.timeRange === 'custom' && (React.createElement("div", { style: styles.customDateRange },
                    React.createElement("input", { type: "datetime-local", value: localFilters.startDate || '', onChange: (e) => handleCustomDateRange(e.target.value, localFilters.endDate || ''), style: styles.dateInput, "aria-label": "Start date" }),
                    React.createElement("span", { style: styles.dateSeparator }, "to"),
                    React.createElement("input", { type: "datetime-local", value: localFilters.endDate || '', onChange: (e) => handleCustomDateRange(localFilters.startDate || '', e.target.value), style: styles.dateInput, "aria-label": "End date" }))))),
            showDepartments && departments.length > 0 && (React.createElement("div", { style: styles.filterGroup },
                React.createElement("label", { style: styles.label, id: "departments-label" }, "Departments"),
                React.createElement("div", { style: styles.checkboxGroup, role: "group", "aria-labelledby": "departments-label" }, departments.map((dept) => (React.createElement("label", { key: dept, style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: localFilters.departments?.includes(dept) || false, onChange: () => toggleDepartment(dept), style: styles.checkbox, "aria-label": dept }),
                    React.createElement("span", { style: styles.checkboxText }, dept))))))),
            showRegions && regions.length > 0 && (React.createElement("div", { style: styles.filterGroup },
                React.createElement("label", { style: styles.label, id: "regions-label" }, "Regions"),
                React.createElement("div", { style: styles.checkboxGroup, role: "group", "aria-labelledby": "regions-label" }, regions.map((region) => (React.createElement("label", { key: region, style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: localFilters.regions?.includes(region) || false, onChange: () => toggleRegion(region), style: styles.checkbox, "aria-label": region }),
                    React.createElement("span", { style: styles.checkboxText }, region))))))),
            showUsers && users.length > 0 && (React.createElement("div", { style: styles.filterGroup },
                React.createElement("label", { style: styles.label, id: "users-label" }, "Users"),
                React.createElement("div", { style: styles.checkboxGroup, role: "group", "aria-labelledby": "users-label" }, users.map((user) => (React.createElement("label", { key: user, style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: localFilters.users?.includes(user) || false, onChange: () => toggleUser(user), style: styles.checkbox, "aria-label": user }),
                    React.createElement("span", { style: styles.checkboxText }, user))))))),
            showStatuses && statuses.length > 0 && (React.createElement("div", { style: styles.filterGroup },
                React.createElement("label", { style: styles.label, id: "statuses-label" }, "Statuses"),
                React.createElement("div", { style: styles.checkboxGroup, role: "group", "aria-labelledby": "statuses-label" }, statuses.map((status) => (React.createElement("label", { key: status, style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: localFilters.statuses?.includes(status) || false, onChange: () => toggleStatus(status), style: styles.checkbox, "aria-label": status }),
                    React.createElement("span", { style: styles.checkboxText }, status))))))),
            enableSavedFilters && (React.createElement("div", { style: styles.filterGroup },
                React.createElement("label", { style: styles.label, id: "save-filter-label" }, "Save Current Filters"),
                React.createElement("div", { style: styles.saveFilterRow },
                    React.createElement("input", { type: "text", value: filterName, onChange: (e) => setFilterName(e.target.value), placeholder: "Filter name...", style: styles.filterNameInput, "aria-labelledby": "save-filter-label" }),
                    React.createElement("button", { onClick: saveFilters, style: styles.saveButton, disabled: !filterName, "aria-label": "Save filter" }, "Save")))),
            enableSavedFilters && savedFilters.length > 0 && (React.createElement("div", { style: styles.filterGroup },
                React.createElement("label", { style: styles.label }, "Saved Filters"),
                React.createElement("div", { style: styles.savedFiltersList }, savedFilters.map((saved) => (React.createElement("div", { key: saved.id, style: styles.savedFilterItem },
                    React.createElement("button", { onClick: () => loadSavedFilter(saved), style: styles.savedFilterButton, "aria-label": `Load filter: ${saved.name}` }, saved.name),
                    React.createElement("button", { onClick: () => deleteSavedFilter(saved.id), style: styles.deleteFilterButton, "aria-label": `Delete filter: ${saved.name}` }, "\u00D7")))))))))));
};
const styles = {
    container: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        border: '1px solid var(--color-border, #e0e0e0)',
        overflow: 'hidden',
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '12px 16px',
        borderBottom: '1px solid var(--color-divider, #e0e0e0)',
        backgroundColor: 'var(--color-background, #f5f5f5)',
    },
    title: {
        margin: 0,
        fontSize: 16,
        fontWeight: 600,
        color: 'var(--color-text, #333)',
        display: 'flex',
        alignItems: 'center',
        gap: 8,
    },
    activeCount: {
        display: 'inline-block',
        minWidth: 20,
        height: 20,
        padding: '0 6px',
        backgroundColor: 'var(--color-primary, #1976d2)',
        color: '#fff',
        borderRadius: 10,
        fontSize: 12,
        fontWeight: 600,
        lineHeight: '20px',
        textAlign: 'center',
    },
    headerActions: {
        display: 'flex',
        gap: 8,
    },
    expandButton: {
        padding: '4px 12px',
        border: 'none',
        backgroundColor: 'transparent',
        color: 'var(--color-text-secondary, #666)',
        cursor: 'pointer',
        fontSize: 12,
        fontWeight: 500,
    },
    resetButton: {
        padding: '4px 12px',
        border: '1px solid var(--color-border, #e0e0e0)',
        backgroundColor: 'var(--color-surface, #fff)',
        color: 'var(--color-error, #f44336)',
        cursor: 'pointer',
        borderRadius: 4,
        fontSize: 12,
        fontWeight: 500,
    },
    content: {
        padding: 16,
        maxHeight: 500,
        overflowY: 'auto',
    },
    filterGroup: {
        marginBottom: 20,
    },
    label: {
        display: 'block',
        marginBottom: 8,
        fontSize: 14,
        fontWeight: 600,
        color: 'var(--color-text, #333)',
    },
    select: {
        width: '100%',
        padding: '8px 12px',
        border: '1px solid var(--color-border, #e0e0e0)',
        borderRadius: 4,
        fontSize: 14,
        backgroundColor: 'var(--color-surface, #fff)',
        color: 'var(--color-text, #333)',
        cursor: 'pointer',
    },
    customDateRange: {
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        marginTop: 8,
    },
    dateInput: {
        flex: 1,
        padding: '8px 12px',
        border: '1px solid var(--color-border, #e0e0e0)',
        borderRadius: 4,
        fontSize: 13,
        backgroundColor: 'var(--color-surface, #fff)',
        color: 'var(--color-text, #333)',
    },
    dateSeparator: {
        fontSize: 12,
        color: 'var(--color-text-secondary, #666)',
    },
    checkboxGroup: {
        display: 'flex',
        flexDirection: 'column',
        gap: 8,
    },
    checkboxLabel: {
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        cursor: 'pointer',
        fontSize: 14,
    },
    checkbox: {
        width: 16,
        height: 16,
        cursor: 'pointer',
    },
    checkboxText: {
        color: 'var(--color-text, #333)',
    },
    saveFilterRow: {
        display: 'flex',
        gap: 8,
    },
    filterNameInput: {
        flex: 1,
        padding: '8px 12px',
        border: '1px solid var(--color-border, #e0e0e0)',
        borderRadius: 4,
        fontSize: 14,
        backgroundColor: 'var(--color-surface, #fff)',
        color: 'var(--color-text, #333)',
    },
    saveButton: {
        padding: '8px 16px',
        border: 'none',
        backgroundColor: 'var(--color-primary, #1976d2)',
        color: '#fff',
        cursor: 'pointer',
        borderRadius: 4,
        fontSize: 14,
        fontWeight: 500,
    },
    savedFiltersList: {
        display: 'flex',
        flexDirection: 'column',
        gap: 8,
    },
    savedFilterItem: {
        display: 'flex',
        gap: 8,
    },
    savedFilterButton: {
        flex: 1,
        padding: '8px 12px',
        border: '1px solid var(--color-border, #e0e0e0)',
        backgroundColor: 'var(--color-surface, #fff)',
        color: 'var(--color-text, #333)',
        cursor: 'pointer',
        borderRadius: 4,
        fontSize: 13,
        textAlign: 'left',
    },
    deleteFilterButton: {
        width: 32,
        height: 32,
        border: '1px solid var(--color-border, #e0e0e0)',
        backgroundColor: 'var(--color-surface, #fff)',
        color: 'var(--color-error, #f44336)',
        cursor: 'pointer',
        borderRadius: 4,
        fontSize: 20,
        fontWeight: 500,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
    },
};
export default DashboardFiltersComponent;
//# sourceMappingURL=DashboardFilters.js.map