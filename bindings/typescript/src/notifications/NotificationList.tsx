/**
 * CADDY v0.4.0 - Notification List Component
 * Notification feed with filtering and grouping
 */

import React, { useState, useMemo, useCallback } from 'react';
import { NotificationItem } from './NotificationItem';
import {
  NotificationFilter,
  NotificationGroupBy,
  NotificationType,
  NotificationPriority,
  NotificationStatus
} from './types';
import { useNotifications, useFilteredNotifications, useGroupedNotifications, useBatchOperations } from './useNotifications';

interface NotificationListProps {
  initialFilter?: NotificationFilter;
  showFilters?: boolean;
  showGrouping?: boolean;
  showBulkActions?: boolean;
  compact?: boolean;
  maxHeight?: string;
}

export const NotificationList: React.FC<NotificationListProps> = ({
  initialFilter = {},
  showFilters = true,
  showGrouping = true,
  showBulkActions = true,
  compact = false,
  maxHeight = '600px'
}) => {
  const { loading, markAllAsRead, notifications } = useNotifications();
  const [localFilter, setLocalFilter] = useState<NotificationFilter>(initialFilter);
  const [groupBy, setGroupBy] = useState<NotificationGroupBy>(NotificationGroupBy.NONE);
  const [searchQuery, setSearchQuery] = useState('');

  const {
    selectedIds,
    selectedCount,
    toggleSelection,
    selectAll,
    clearSelection,
    markSelectedAsRead,
    markSelectedAsUnread,
    archiveSelected,
    deleteSelected
  } = useBatchOperations();

  const filteredNotifications = useFilteredNotifications({ ...localFilter, search: searchQuery });
  const groupedNotifications = useGroupedNotifications(groupBy);

  const handleFilterChange = useCallback((key: keyof NotificationFilter, value: any) => {
    setLocalFilter(prev => ({ ...prev, [key]: value }));
  }, []);

  const handleTypeFilter = useCallback((type: NotificationType) => {
    setLocalFilter(prev => {
      const types = prev.type || [];
      const newTypes = types.includes(type)
        ? types.filter(t => t !== type)
        : [...types, type];
      return { ...prev, type: newTypes.length > 0 ? newTypes : undefined };
    });
  }, []);

  const handlePriorityFilter = useCallback((priority: NotificationPriority) => {
    setLocalFilter(prev => {
      const priorities = prev.priority || [];
      const newPriorities = priorities.includes(priority)
        ? priorities.filter(p => p !== priority)
        : [...priorities, priority];
      return { ...prev, priority: newPriorities.length > 0 ? newPriorities : undefined };
    });
  }, []);

  const handleStatusFilter = useCallback((status: NotificationStatus) => {
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
    } else {
      selectAll(filteredNotifications.map(n => n.id));
    }
  }, [selectedCount, filteredNotifications, clearSelection, selectAll]);

  const hasActiveFilters = useMemo(() => {
    return !!(
      localFilter.type?.length ||
      localFilter.priority?.length ||
      localFilter.status?.length ||
      localFilter.unreadOnly ||
      searchQuery
    );
  }, [localFilter, searchQuery]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', backgroundColor: '#ffffff' }}>
      {/* Header */}
      <div style={{ padding: '16px', borderBottom: '1px solid #e5e7eb' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
          <h2 style={{ margin: 0, fontSize: '20px', fontWeight: '600', color: '#111827' }}>
            Notifications
          </h2>
          {filteredNotifications.length > 0 && (
            <button
              onClick={markAllAsRead}
              style={{
                padding: '6px 12px',
                fontSize: '13px',
                fontWeight: '500',
                border: '1px solid #d1d5db',
                borderRadius: '6px',
                backgroundColor: '#ffffff',
                color: '#374151',
                cursor: 'pointer'
              }}
            >
              Mark all as read
            </button>
          )}
        </div>

        {/* Search */}
        <input
          type="text"
          placeholder="Search notifications..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          style={{
            width: '100%',
            padding: '8px 12px',
            fontSize: '14px',
            border: '1px solid #d1d5db',
            borderRadius: '6px',
            outline: 'none',
            transition: 'border-color 0.2s'
          }}
          onFocus={(e) => {
            e.currentTarget.style.borderColor = '#3b82f6';
          }}
          onBlur={(e) => {
            e.currentTarget.style.borderColor = '#d1d5db';
          }}
        />
      </div>

      {/* Filters */}
      {showFilters && (
        <div style={{ padding: '12px 16px', borderBottom: '1px solid #e5e7eb', backgroundColor: '#f9fafb' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {/* Quick filters */}
            <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', alignItems: 'center' }}>
              <span style={{ fontSize: '12px', fontWeight: '500', color: '#6b7280' }}>Type:</span>
              {Object.values(NotificationType).map((type) => (
                <button
                  key={type}
                  onClick={() => handleTypeFilter(type)}
                  style={{
                    padding: '4px 8px',
                    fontSize: '11px',
                    fontWeight: '500',
                    border: '1px solid #d1d5db',
                    borderRadius: '4px',
                    backgroundColor: localFilter.type?.includes(type) ? '#3b82f6' : '#ffffff',
                    color: localFilter.type?.includes(type) ? '#ffffff' : '#374151',
                    cursor: 'pointer',
                    textTransform: 'capitalize'
                  }}
                >
                  {type}
                </button>
              ))}
            </div>

            <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', alignItems: 'center' }}>
              <span style={{ fontSize: '12px', fontWeight: '500', color: '#6b7280' }}>Priority:</span>
              {Object.values(NotificationPriority).map((priority) => (
                <button
                  key={priority}
                  onClick={() => handlePriorityFilter(priority)}
                  style={{
                    padding: '4px 8px',
                    fontSize: '11px',
                    fontWeight: '500',
                    border: '1px solid #d1d5db',
                    borderRadius: '4px',
                    backgroundColor: localFilter.priority?.includes(priority) ? '#3b82f6' : '#ffffff',
                    color: localFilter.priority?.includes(priority) ? '#ffffff' : '#374151',
                    cursor: 'pointer',
                    textTransform: 'capitalize'
                  }}
                >
                  {priority}
                </button>
              ))}
            </div>

            <div style={{ display: 'flex', gap: '12px', alignItems: 'center', flexWrap: 'wrap' }}>
              <label style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px', color: '#374151' }}>
                <input
                  type="checkbox"
                  checked={localFilter.unreadOnly || false}
                  onChange={(e) => handleFilterChange('unreadOnly', e.target.checked || undefined)}
                  style={{ cursor: 'pointer' }}
                />
                Unread only
              </label>

              {showGrouping && (
                <select
                  value={groupBy}
                  onChange={(e) => setGroupBy(e.target.value as NotificationGroupBy)}
                  style={{
                    padding: '4px 8px',
                    fontSize: '12px',
                    border: '1px solid #d1d5db',
                    borderRadius: '4px',
                    backgroundColor: '#ffffff',
                    color: '#374151',
                    cursor: 'pointer'
                  }}
                >
                  <option value={NotificationGroupBy.NONE}>No grouping</option>
                  <option value={NotificationGroupBy.TYPE}>Group by type</option>
                  <option value={NotificationGroupBy.PRIORITY}>Group by priority</option>
                  <option value={NotificationGroupBy.SOURCE}>Group by source</option>
                  <option value={NotificationGroupBy.DATE}>Group by date</option>
                </select>
              )}

              {hasActiveFilters && (
                <button
                  onClick={handleClearFilters}
                  style={{
                    padding: '4px 8px',
                    fontSize: '12px',
                    color: '#dc2626',
                    background: 'none',
                    border: 'none',
                    cursor: 'pointer',
                    textDecoration: 'underline'
                  }}
                >
                  Clear filters
                </button>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Bulk actions */}
      {showBulkActions && selectedCount > 0 && (
        <div style={{ padding: '12px 16px', backgroundColor: '#eff6ff', borderBottom: '1px solid #bfdbfe' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <span style={{ fontSize: '13px', color: '#1e40af', fontWeight: '500' }}>
              {selectedCount} selected
            </span>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button
                onClick={markSelectedAsRead}
                style={{
                  padding: '4px 12px',
                  fontSize: '12px',
                  fontWeight: '500',
                  border: '1px solid #3b82f6',
                  borderRadius: '4px',
                  backgroundColor: '#ffffff',
                  color: '#3b82f6',
                  cursor: 'pointer'
                }}
              >
                Mark as read
              </button>
              <button
                onClick={archiveSelected}
                style={{
                  padding: '4px 12px',
                  fontSize: '12px',
                  fontWeight: '500',
                  border: '1px solid #3b82f6',
                  borderRadius: '4px',
                  backgroundColor: '#ffffff',
                  color: '#3b82f6',
                  cursor: 'pointer'
                }}
              >
                Archive
              </button>
              <button
                onClick={deleteSelected}
                style={{
                  padding: '4px 12px',
                  fontSize: '12px',
                  fontWeight: '500',
                  border: '1px solid #dc2626',
                  borderRadius: '4px',
                  backgroundColor: '#ffffff',
                  color: '#dc2626',
                  cursor: 'pointer'
                }}
              >
                Delete
              </button>
              <button
                onClick={clearSelection}
                style={{
                  padding: '4px 12px',
                  fontSize: '12px',
                  fontWeight: '500',
                  border: 'none',
                  borderRadius: '4px',
                  backgroundColor: 'transparent',
                  color: '#6b7280',
                  cursor: 'pointer'
                }}
              >
                Clear
              </button>
            </div>
          </div>
        </div>
      )}

      {/* List */}
      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          maxHeight
        }}
      >
        {loading && filteredNotifications.length === 0 ? (
          <div style={{ padding: '48px 16px', textAlign: 'center', color: '#6b7280' }}>
            <div style={{ fontSize: '14px' }}>Loading notifications...</div>
          </div>
        ) : filteredNotifications.length === 0 ? (
          <div style={{ padding: '48px 16px', textAlign: 'center', color: '#6b7280' }}>
            <div style={{ fontSize: '48px', marginBottom: '16px' }}>🔔</div>
            <div style={{ fontSize: '16px', fontWeight: '500', marginBottom: '8px' }}>No notifications</div>
            <div style={{ fontSize: '14px' }}>
              {hasActiveFilters ? 'Try adjusting your filters' : 'You\'re all caught up!'}
            </div>
          </div>
        ) : groupBy !== NotificationGroupBy.NONE && groupedNotifications.length > 0 ? (
          // Grouped view
          <div>
            {groupedNotifications.map((group) => (
              <div key={group.id}>
                <div
                  style={{
                    padding: '12px 16px',
                    backgroundColor: '#f9fafb',
                    borderBottom: '1px solid #e5e7eb',
                    fontSize: '13px',
                    fontWeight: '600',
                    color: '#374151',
                    textTransform: 'capitalize'
                  }}
                >
                  {group.id} ({group.count})
                </div>
                {group.notifications.map((notification) => (
                  <NotificationItem
                    key={notification.id}
                    notification={notification}
                    compact={compact}
                    onSelect={showBulkActions ? toggleSelection : undefined}
                    selected={selectedIds.has(notification.id)}
                  />
                ))}
              </div>
            ))}
          </div>
        ) : (
          // Flat view
          <div>
            {showBulkActions && filteredNotifications.length > 0 && (
              <div
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#f9fafb',
                  borderBottom: '1px solid #e5e7eb',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px'
                }}
              >
                <input
                  type="checkbox"
                  checked={selectedCount === filteredNotifications.length && selectedCount > 0}
                  onChange={handleSelectAll}
                  style={{ cursor: 'pointer' }}
                />
                <span style={{ fontSize: '12px', color: '#6b7280' }}>
                  Select all ({filteredNotifications.length})
                </span>
              </div>
            )}
            {filteredNotifications.map((notification) => (
              <NotificationItem
                key={notification.id}
                notification={notification}
                compact={compact}
                onSelect={showBulkActions ? toggleSelection : undefined}
                selected={selectedIds.has(notification.id)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
