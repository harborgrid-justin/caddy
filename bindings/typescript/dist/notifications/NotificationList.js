import React, { useState, useMemo, useCallback } from 'react';
import { NotificationItem } from './NotificationItem';
import { NotificationGroupBy, NotificationType, NotificationPriority } from './types';
import { useNotifications, useFilteredNotifications, useGroupedNotifications, useBatchOperations } from './useNotifications';
export const NotificationList = ({ initialFilter = {}, showFilters = true, showGrouping = true, showBulkActions = true, compact = false, maxHeight = '600px' }) => {
    const { loading, markAllAsRead, notifications } = useNotifications();
    const [localFilter, setLocalFilter] = useState(initialFilter);
    const [groupBy, setGroupBy] = useState(NotificationGroupBy.NONE);
    const [searchQuery, setSearchQuery] = useState('');
    const { selectedIds, selectedCount, toggleSelection, selectAll, clearSelection, markSelectedAsRead, markSelectedAsUnread, archiveSelected, deleteSelected } = useBatchOperations();
    const filteredNotifications = useFilteredNotifications({ ...localFilter, search: searchQuery });
    const groupedNotifications = useGroupedNotifications(groupBy);
    const handleFilterChange = useCallback((key, value) => {
        setLocalFilter(prev => ({ ...prev, [key]: value }));
    }, []);
    const handleTypeFilter = useCallback((type) => {
        setLocalFilter(prev => {
            const types = prev.type || [];
            const newTypes = types.includes(type)
                ? types.filter(t => t !== type)
                : [...types, type];
            return { ...prev, type: newTypes.length > 0 ? newTypes : undefined };
        });
    }, []);
    const handlePriorityFilter = useCallback((priority) => {
        setLocalFilter(prev => {
            const priorities = prev.priority || [];
            const newPriorities = priorities.includes(priority)
                ? priorities.filter(p => p !== priority)
                : [...priorities, priority];
            return { ...prev, priority: newPriorities.length > 0 ? newPriorities : undefined };
        });
    }, []);
    const handleStatusFilter = useCallback((status) => {
        setLocalFilter(prev => {
            const statuses = prev.status || [];
            const newStatuses = statuses.includes(status)
                ? statuses.filter(s => s !== status)
                : [...statuses, status];
            return { ...prev, status: newStatuses.length > 0 ? newStatuses : undefined };
        });
    }, []);
    const handleClearFilters = useCallback(() => {
        setLocalFilter({});
        setSearchQuery('');
        setGroupBy(NotificationGroupBy.NONE);
    }, []);
    const handleSelectAll = useCallback(() => {
        if (selectedCount === filteredNotifications.length) {
            clearSelection();
        }
        else {
            selectAll(filteredNotifications.map(n => n.id));
        }
    }, [selectedCount, filteredNotifications, clearSelection, selectAll]);
    const hasActiveFilters = useMemo(() => {
        return !!(localFilter.type?.length ||
            localFilter.priority?.length ||
            localFilter.status?.length ||
            localFilter.unreadOnly ||
            searchQuery);
    }, [localFilter, searchQuery]);
    return (React.createElement("div", { style: { display: 'flex', flexDirection: 'column', height: '100%', backgroundColor: '#ffffff' } },
        React.createElement("div", { style: { padding: '16px', borderBottom: '1px solid #e5e7eb' } },
            React.createElement("div", { style: { display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' } },
                React.createElement("h2", { style: { margin: 0, fontSize: '20px', fontWeight: '600', color: '#111827' } }, "Notifications"),
                filteredNotifications.length > 0 && (React.createElement("button", { onClick: markAllAsRead, style: {
                        padding: '6px 12px',
                        fontSize: '13px',
                        fontWeight: '500',
                        border: '1px solid #d1d5db',
                        borderRadius: '6px',
                        backgroundColor: '#ffffff',
                        color: '#374151',
                        cursor: 'pointer'
                    } }, "Mark all as read"))),
            React.createElement("input", { type: "text", placeholder: "Search notifications...", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value), style: {
                    width: '100%',
                    padding: '8px 12px',
                    fontSize: '14px',
                    border: '1px solid #d1d5db',
                    borderRadius: '6px',
                    outline: 'none',
                    transition: 'border-color 0.2s'
                }, onFocus: (e) => {
                    e.currentTarget.style.borderColor = '#3b82f6';
                }, onBlur: (e) => {
                    e.currentTarget.style.borderColor = '#d1d5db';
                } })),
        showFilters && (React.createElement("div", { style: { padding: '12px 16px', borderBottom: '1px solid #e5e7eb', backgroundColor: '#f9fafb' } },
            React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '12px' } },
                React.createElement("div", { style: { display: 'flex', gap: '8px', flexWrap: 'wrap', alignItems: 'center' } },
                    React.createElement("span", { style: { fontSize: '12px', fontWeight: '500', color: '#6b7280' } }, "Type:"),
                    Object.values(NotificationType).map((type) => (React.createElement("button", { key: type, onClick: () => handleTypeFilter(type), style: {
                            padding: '4px 8px',
                            fontSize: '11px',
                            fontWeight: '500',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: localFilter.type?.includes(type) ? '#3b82f6' : '#ffffff',
                            color: localFilter.type?.includes(type) ? '#ffffff' : '#374151',
                            cursor: 'pointer',
                            textTransform: 'capitalize'
                        } }, type)))),
                React.createElement("div", { style: { display: 'flex', gap: '8px', flexWrap: 'wrap', alignItems: 'center' } },
                    React.createElement("span", { style: { fontSize: '12px', fontWeight: '500', color: '#6b7280' } }, "Priority:"),
                    Object.values(NotificationPriority).map((priority) => (React.createElement("button", { key: priority, onClick: () => handlePriorityFilter(priority), style: {
                            padding: '4px 8px',
                            fontSize: '11px',
                            fontWeight: '500',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: localFilter.priority?.includes(priority) ? '#3b82f6' : '#ffffff',
                            color: localFilter.priority?.includes(priority) ? '#ffffff' : '#374151',
                            cursor: 'pointer',
                            textTransform: 'capitalize'
                        } }, priority)))),
                React.createElement("div", { style: { display: 'flex', gap: '12px', alignItems: 'center', flexWrap: 'wrap' } },
                    React.createElement("label", { style: { display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px', color: '#374151' } },
                        React.createElement("input", { type: "checkbox", checked: localFilter.unreadOnly || false, onChange: (e) => handleFilterChange('unreadOnly', e.target.checked || undefined), style: { cursor: 'pointer' } }),
                        "Unread only"),
                    showGrouping && (React.createElement("select", { value: groupBy, onChange: (e) => setGroupBy(e.target.value), style: {
                            padding: '4px 8px',
                            fontSize: '12px',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#374151',
                            cursor: 'pointer'
                        } },
                        React.createElement("option", { value: NotificationGroupBy.NONE }, "No grouping"),
                        React.createElement("option", { value: NotificationGroupBy.TYPE }, "Group by type"),
                        React.createElement("option", { value: NotificationGroupBy.PRIORITY }, "Group by priority"),
                        React.createElement("option", { value: NotificationGroupBy.SOURCE }, "Group by source"),
                        React.createElement("option", { value: NotificationGroupBy.DATE }, "Group by date"))),
                    hasActiveFilters && (React.createElement("button", { onClick: handleClearFilters, style: {
                            padding: '4px 8px',
                            fontSize: '12px',
                            color: '#dc2626',
                            background: 'none',
                            border: 'none',
                            cursor: 'pointer',
                            textDecoration: 'underline'
                        } }, "Clear filters")))))),
        showBulkActions && selectedCount > 0 && (React.createElement("div", { style: { padding: '12px 16px', backgroundColor: '#eff6ff', borderBottom: '1px solid #bfdbfe' } },
            React.createElement("div", { style: { display: 'flex', alignItems: 'center', justifyContent: 'space-between' } },
                React.createElement("span", { style: { fontSize: '13px', color: '#1e40af', fontWeight: '500' } },
                    selectedCount,
                    " selected"),
                React.createElement("div", { style: { display: 'flex', gap: '8px' } },
                    React.createElement("button", { onClick: markSelectedAsRead, style: {
                            padding: '4px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: '1px solid #3b82f6',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#3b82f6',
                            cursor: 'pointer'
                        } }, "Mark as read"),
                    React.createElement("button", { onClick: archiveSelected, style: {
                            padding: '4px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: '1px solid #3b82f6',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#3b82f6',
                            cursor: 'pointer'
                        } }, "Archive"),
                    React.createElement("button", { onClick: deleteSelected, style: {
                            padding: '4px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: '1px solid #dc2626',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#dc2626',
                            cursor: 'pointer'
                        } }, "Delete"),
                    React.createElement("button", { onClick: clearSelection, style: {
                            padding: '4px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: 'none',
                            borderRadius: '4px',
                            backgroundColor: 'transparent',
                            color: '#6b7280',
                            cursor: 'pointer'
                        } }, "Clear"))))),
        React.createElement("div", { style: {
                flex: 1,
                overflowY: 'auto',
                maxHeight
            } }, loading && filteredNotifications.length === 0 ? (React.createElement("div", { style: { padding: '48px 16px', textAlign: 'center', color: '#6b7280' } },
            React.createElement("div", { style: { fontSize: '14px' } }, "Loading notifications..."))) : filteredNotifications.length === 0 ? (React.createElement("div", { style: { padding: '48px 16px', textAlign: 'center', color: '#6b7280' } },
            React.createElement("div", { style: { fontSize: '48px', marginBottom: '16px' } }, "\uD83D\uDD14"),
            React.createElement("div", { style: { fontSize: '16px', fontWeight: '500', marginBottom: '8px' } }, "No notifications"),
            React.createElement("div", { style: { fontSize: '14px' } }, hasActiveFilters ? 'Try adjusting your filters' : 'You\'re all caught up!'))) : groupBy !== NotificationGroupBy.NONE && groupedNotifications.length > 0 ? (React.createElement("div", null, groupedNotifications.map((group) => (React.createElement("div", { key: group.id },
            React.createElement("div", { style: {
                    padding: '12px 16px',
                    backgroundColor: '#f9fafb',
                    borderBottom: '1px solid #e5e7eb',
                    fontSize: '13px',
                    fontWeight: '600',
                    color: '#374151',
                    textTransform: 'capitalize'
                } },
                group.id,
                " (",
                group.count,
                ")"),
            group.notifications.map((notification) => (React.createElement(NotificationItem, { key: notification.id, notification: notification, compact: compact, onSelect: showBulkActions ? toggleSelection : undefined, selected: selectedIds.has(notification.id) })))))))) : (React.createElement("div", null,
            showBulkActions && filteredNotifications.length > 0 && (React.createElement("div", { style: {
                    padding: '8px 16px',
                    backgroundColor: '#f9fafb',
                    borderBottom: '1px solid #e5e7eb',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px'
                } },
                React.createElement("input", { type: "checkbox", checked: selectedCount === filteredNotifications.length && selectedCount > 0, onChange: handleSelectAll, style: { cursor: 'pointer' } }),
                React.createElement("span", { style: { fontSize: '12px', color: '#6b7280' } },
                    "Select all (",
                    filteredNotifications.length,
                    ")"))),
            filteredNotifications.map((notification) => (React.createElement(NotificationItem, { key: notification.id, notification: notification, compact: compact, onSelect: showBulkActions ? toggleSelection : undefined, selected: selectedIds.has(notification.id) }))))))));
};
//# sourceMappingURL=NotificationList.js.map