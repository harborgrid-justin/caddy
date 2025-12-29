/**
 * Audit Log Component
 * Searchable audit log viewer with advanced filtering and pagination
 */

import React, { useState, useEffect, useCallback } from 'react';
import { AuditFilters } from './AuditFilters';
import type { AuditEvent, AuditFilter, AuditSeverity, AuditStatus } from './types';

interface AuditLogProps {
  organizationId?: string;
  defaultFilters?: AuditFilter;
  onEventSelect?: (event: AuditEvent) => void;
}

export const AuditLog: React.FC<AuditLogProps> = ({
  organizationId,
  defaultFilters = {},
  onEventSelect,
}) => {
  const [events, setEvents] = useState<AuditEvent[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [loading, setLoading] = useState(false);
  const [filters, setFilters] = useState<AuditFilter>(defaultFilters);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(50);
  const [selectedEvents, setSelectedEvents] = useState<Set<string>>(new Set());
  const [sortField, setSortField] = useState<keyof AuditEvent>('timestamp');
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('desc');

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

      // Add filter parameters
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
    } catch (error) {
      console.error('Failed to load audit events:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleFilterChange = useCallback((newFilters: AuditFilter) => {
    setFilters(newFilters);
    setPage(1); // Reset to first page when filters change
  }, []);

  const handleSort = (field: keyof AuditEvent) => {
    if (field === sortField) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDirection('desc');
    }
  };

  const handleSelectEvent = (eventId: string) => {
    const newSelected = new Set(selectedEvents);
    if (newSelected.has(eventId)) {
      newSelected.delete(eventId);
    } else {
      newSelected.add(eventId);
    }
    setSelectedEvents(newSelected);
  };

  const handleSelectAll = () => {
    if (selectedEvents.size === events.length) {
      setSelectedEvents(new Set());
    } else {
      setSelectedEvents(new Set(events.map((e) => e.id)));
    }
  };

  const handleBulkExport = async () => {
    const selectedEventsList = events.filter((e) => selectedEvents.has(e.id));
    const csvContent = generateCSV(selectedEventsList);
    downloadFile(csvContent, 'audit-logs.csv', 'text/csv');
  };

  const totalPages = Math.ceil(totalCount / pageSize);

  return (
    <div className="audit-log">
      {/* Header */}
      <div className="audit-log-header">
        <div>
          <h2>Audit Logs</h2>
          <p className="subtitle">
            {totalCount.toLocaleString()} events found
          </p>
        </div>
        <div className="header-actions">
          <button
            className="btn btn-secondary"
            onClick={handleBulkExport}
            disabled={selectedEvents.size === 0}
          >
            Export Selected ({selectedEvents.size})
          </button>
          <button className="btn btn-primary" onClick={loadEvents}>
            Refresh
          </button>
        </div>
      </div>

      {/* Filters */}
      <AuditFilters
        filters={filters}
        onChange={handleFilterChange}
        onReset={() => handleFilterChange({})}
      />

      {/* Table Controls */}
      <div className="table-controls">
        <div className="page-size-selector">
          <label>Show:</label>
          <select
            value={pageSize}
            onChange={(e) => {
              setPageSize(Number(e.target.value));
              setPage(1);
            }}
          >
            <option value={25}>25</option>
            <option value={50}>50</option>
            <option value={100}>100</option>
            <option value={250}>250</option>
          </select>
          <span>per page</span>
        </div>

        <div className="view-options">
          <button
            className={`view-btn ${selectedEvents.size === 0 ? 'active' : ''}`}
            onClick={() => setSelectedEvents(new Set())}
          >
            All Events
          </button>
          <button
            className={`view-btn ${filters.anomaly_only ? 'active' : ''}`}
            onClick={() =>
              handleFilterChange({ ...filters, anomaly_only: !filters.anomaly_only })
            }
          >
            Anomalies Only
          </button>
        </div>
      </div>

      {/* Events Table */}
      <div className="table-container">
        {loading ? (
          <div className="loading-state">
            <div className="loading-spinner" />
            <p>Loading audit events...</p>
          </div>
        ) : events.length === 0 ? (
          <div className="empty-state">
            <p>No audit events found matching your filters</p>
            <button onClick={() => handleFilterChange({})}>
              Clear Filters
            </button>
          </div>
        ) : (
          <table className="audit-table">
            <thead>
              <tr>
                <th className="checkbox-cell">
                  <input
                    type="checkbox"
                    checked={selectedEvents.size === events.length}
                    onChange={handleSelectAll}
                  />
                </th>
                <th onClick={() => handleSort('timestamp')} className="sortable">
                  Timestamp
                  {sortField === 'timestamp' && (
                    <SortIndicator direction={sortDirection} />
                  )}
                </th>
                <th onClick={() => handleSort('event_type')} className="sortable">
                  Event Type
                  {sortField === 'event_type' && (
                    <SortIndicator direction={sortDirection} />
                  )}
                </th>
                <th onClick={() => handleSort('user_email')} className="sortable">
                  User
                  {sortField === 'user_email' && (
                    <SortIndicator direction={sortDirection} />
                  )}
                </th>
                <th>Resource</th>
                <th>Action</th>
                <th onClick={() => handleSort('status')} className="sortable">
                  Status
                  {sortField === 'status' && (
                    <SortIndicator direction={sortDirection} />
                  )}
                </th>
                <th onClick={() => handleSort('severity')} className="sortable">
                  Severity
                  {sortField === 'severity' && (
                    <SortIndicator direction={sortDirection} />
                  )}
                </th>
                <th>Risk</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {events.map((event) => (
                <tr
                  key={event.id}
                  className={`${event.anomaly_detected ? 'anomaly-row' : ''} ${
                    selectedEvents.has(event.id) ? 'selected' : ''
                  }`}
                >
                  <td className="checkbox-cell">
                    <input
                      type="checkbox"
                      checked={selectedEvents.has(event.id)}
                      onChange={() => handleSelectEvent(event.id)}
                    />
                  </td>
                  <td className="timestamp-cell">
                    {formatTimestamp(event.timestamp)}
                  </td>
                  <td className="event-type-cell">
                    <div className="event-type-wrapper">
                      {formatEventType(event.event_type)}
                      {event.anomaly_detected && (
                        <span className="anomaly-badge" title="Anomaly detected">
                          !
                        </span>
                      )}
                    </div>
                  </td>
                  <td>
                    <div className="user-info">
                      <div className="user-email">
                        {event.user_email || 'System'}
                      </div>
                      <div className="user-ip">{event.user_ip_address}</div>
                    </div>
                  </td>
                  <td>
                    {event.resource_name || event.resource_id ? (
                      <div className="resource-info">
                        <div className="resource-name">
                          {event.resource_name || event.resource_id}
                        </div>
                        {event.resource_type && (
                          <div className="resource-type">{event.resource_type}</div>
                        )}
                      </div>
                    ) : (
                      '-'
                    )}
                  </td>
                  <td>{event.action}</td>
                  <td>
                    <StatusBadge status={event.status} />
                  </td>
                  <td>
                    <SeverityBadge severity={event.severity} />
                  </td>
                  <td>
                    {event.risk_score !== undefined && (
                      <RiskScoreBadge score={event.risk_score} />
                    )}
                  </td>
                  <td className="actions-cell">
                    <button
                      className="btn-icon"
                      onClick={() => onEventSelect?.(event)}
                      title="View Details"
                    >
                      👁️
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="pagination">
          <button
            className="pagination-btn"
            onClick={() => setPage(1)}
            disabled={page === 1}
          >
            First
          </button>
          <button
            className="pagination-btn"
            onClick={() => setPage(page - 1)}
            disabled={page === 1}
          >
            Previous
          </button>

          <div className="page-numbers">
            {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
              const pageNum = Math.max(1, Math.min(page - 2 + i, totalPages - 4 + i));
              return (
                <button
                  key={pageNum}
                  className={`page-number ${page === pageNum ? 'active' : ''}`}
                  onClick={() => setPage(pageNum)}
                >
                  {pageNum}
                </button>
              );
            })}
          </div>

          <button
            className="pagination-btn"
            onClick={() => setPage(page + 1)}
            disabled={page === totalPages}
          >
            Next
          </button>
          <button
            className="pagination-btn"
            onClick={() => setPage(totalPages)}
            disabled={page === totalPages}
          >
            Last
          </button>

          <div className="page-info">
            Page {page} of {totalPages}
          </div>
        </div>
      )}
    </div>
  );
};

// Sort Indicator Component
function SortIndicator({ direction }: { direction: 'asc' | 'desc' }) {
  return <span className="sort-indicator">{direction === 'asc' ? '▲' : '▼'}</span>;
}

// Status Badge Component
function StatusBadge({ status }: { status: AuditStatus }) {
  const colors: Record<AuditStatus, string> = {
    success: 'green',
    failure: 'red',
    pending: 'yellow',
    blocked: 'gray',
  };

  return (
    <span className={`badge badge-${colors[status]}`}>
      {status}
    </span>
  );
}

// Severity Badge Component
function SeverityBadge({ severity }: { severity: AuditSeverity }) {
  const colors: Record<AuditSeverity, string> = {
    low: 'green',
    medium: 'yellow',
    high: 'orange',
    critical: 'red',
  };

  return (
    <span className={`badge badge-${colors[severity]}`}>
      {severity.toUpperCase()}
    </span>
  );
}

// Risk Score Badge Component
function RiskScoreBadge({ score }: { score: number }) {
  const getColor = (score: number) => {
    if (score >= 80) return 'red';
    if (score >= 60) return 'orange';
    if (score >= 40) return 'yellow';
    return 'green';
  };

  return (
    <div className="risk-score">
      <span className={`risk-badge badge-${getColor(score)}`}>
        {score.toFixed(0)}
      </span>
    </div>
  );
}

// Utility Functions
function formatTimestamp(timestamp: string): string {
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

function formatEventType(eventType: string): string {
  return eventType
    .split('.')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

function generateCSV(events: AuditEvent[]): string {
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
    ...rows.map((row) =>
      row.map((cell) => `"${cell.toString().replace(/"/g, '""')}"`).join(',')
    ),
  ].join('\n');
}

function downloadFile(content: string, filename: string, mimeType: string) {
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
