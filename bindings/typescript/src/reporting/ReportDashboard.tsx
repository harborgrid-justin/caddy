/**
 * CADDY v0.4.0 - Report Dashboard Component
 * $650M Platform - Production Ready
 *
 * Comprehensive report management dashboard with search, filtering,
 * bulk operations, and advanced report lifecycle management.
 */

import React, { useState, useCallback, useMemo } from 'react';
import {
  ReportDefinition,
  ReportExecution,
  ReportStatus,
} from './types';

export interface ReportDashboardProps {
  reports: ReportDefinition[];
  executions?: ReportExecution[];
  onCreateReport?: () => void;
  onEditReport?: (report: ReportDefinition) => void;
  onDeleteReport?: (reportId: string) => void;
  onDuplicateReport?: (report: ReportDefinition) => void;
  onExecuteReport?: (reportId: string) => void;
  onScheduleReport?: (reportId: string) => void;
  onViewReport?: (reportId: string) => void;
  onExportReport?: (reportId: string) => void;
}

export const ReportDashboard: React.FC<ReportDashboardProps> = ({
  reports,
  executions = [],
  onCreateReport,
  onEditReport,
  onDeleteReport,
  onDuplicateReport,
  onExecuteReport,
  onScheduleReport,
  onViewReport,
  onExportReport,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [statusFilter, setStatusFilter] = useState<ReportStatus | 'all'>('all');
  const [categoryFilter, setCategoryFilter] = useState<string>('all');
  const [sortBy, setSortBy] = useState<'name' | 'updated' | 'created'>('updated');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [selectedReports, setSelectedReports] = useState<Set<string>>(new Set());

  // Get unique categories
  const categories = useMemo(
    () => ['all', ...new Set(reports.map((r) => r.metadata.category).filter(Boolean))],
    [reports]
  );

  // Filter and sort reports
  const filteredReports = useMemo(() => {
    return reports
      .filter((report) => {
        const matchesSearch =
          report.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
          report.description?.toLowerCase().includes(searchTerm.toLowerCase()) ||
          report.metadata.tags?.some((tag) => tag.toLowerCase().includes(searchTerm.toLowerCase()));

        const matchesStatus = statusFilter === 'all' || report.status === statusFilter;

        const matchesCategory =
          categoryFilter === 'all' || report.metadata.category === categoryFilter;

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

  // Toggle report selection
  const toggleReportSelection = useCallback((reportId: string) => {
    setSelectedReports((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(reportId)) {
        newSet.delete(reportId);
      } else {
        newSet.add(reportId);
      }
      return newSet;
    });
  }, []);

  // Select all / deselect all
  const toggleSelectAll = useCallback(() => {
    if (selectedReports.size === filteredReports.length) {
      setSelectedReports(new Set());
    } else {
      setSelectedReports(new Set(filteredReports.map((r) => r.id)));
    }
  }, [selectedReports.size, filteredReports]);

  // Bulk delete
  const handleBulkDelete = useCallback(() => {
    if (!onDeleteReport) return;

    const confirmed = window.confirm(
      `Are you sure you want to delete ${selectedReports.size} report(s)?`
    );

    if (confirmed) {
      selectedReports.forEach((reportId) => onDeleteReport(reportId));
      setSelectedReports(new Set());
    }
  }, [selectedReports, onDeleteReport]);

  // Get last execution for report
  const getLastExecution = useCallback(
    (reportId: string): ReportExecution | undefined => {
      return executions
        .filter((e) => e.reportId === reportId)
        .sort((a, b) => new Date(b.startTime).getTime() - new Date(a.startTime).getTime())[0];
    },
    [executions]
  );

  // Render report card
  const renderReportCard = (report: ReportDefinition) => {
    const lastExecution = getLastExecution(report.id);
    const isSelected = selectedReports.has(report.id);

    return (
      <div
        key={report.id}
        style={{
          ...styles.reportCard,
          ...(isSelected ? styles.reportCardSelected : {}),
        }}
      >
        <div style={styles.reportCardHeader}>
          <input
            type="checkbox"
            checked={isSelected}
            onChange={() => toggleReportSelection(report.id)}
            style={styles.checkbox}
          />
          <span style={getStatusStyle(report.status)}>{report.status}</span>
        </div>

        <div style={styles.reportCardContent}>
          <h3 style={styles.reportName}>{report.name}</h3>
          {report.description && (
            <p style={styles.reportDescription}>{report.description}</p>
          )}

          <div style={styles.reportMeta}>
            <div style={styles.metaItem}>
              <span style={styles.metaLabel}>Type:</span>
              <span>{report.type}</span>
            </div>
            <div style={styles.metaItem}>
              <span style={styles.metaLabel}>Version:</span>
              <span>v{report.version}</span>
            </div>
            {report.metadata.category && (
              <div style={styles.metaItem}>
                <span style={styles.metaLabel}>Category:</span>
                <span>{report.metadata.category}</span>
              </div>
            )}
          </div>

          {lastExecution && (
            <div style={styles.lastExecution}>
              <span style={styles.executionLabel}>Last run:</span>
              <span style={styles.executionTime}>
                {new Date(lastExecution.startTime).toLocaleString()}
              </span>
              <span style={getExecutionStatusStyle(lastExecution.status)}>
                {lastExecution.status}
              </span>
            </div>
          )}

          {report.metadata.tags && report.metadata.tags.length > 0 && (
            <div style={styles.tags}>
              {report.metadata.tags.slice(0, 3).map((tag, index) => (
                <span key={index} style={styles.tag}>
                  {tag}
                </span>
              ))}
              {report.metadata.tags.length > 3 && (
                <span style={styles.tagMore}>+{report.metadata.tags.length - 3}</span>
              )}
            </div>
          )}
        </div>

        <div style={styles.reportCardActions}>
          {onViewReport && (
            <button onClick={() => onViewReport(report.id)} style={styles.actionButton}>
              👁️ View
            </button>
          )}
          {onEditReport && (
            <button onClick={() => onEditReport(report)} style={styles.actionButton}>
              ✎ Edit
            </button>
          )}
          {onExecuteReport && (
            <button onClick={() => onExecuteReport(report.id)} style={styles.actionButton}>
              ▶ Run
            </button>
          )}
          <div style={styles.moreMenu}>
            <button style={styles.moreButton}>⋮</button>
            <div style={styles.moreMenuContent}>
              {onDuplicateReport && (
                <button onClick={() => onDuplicateReport(report)}>Duplicate</button>
              )}
              {onScheduleReport && (
                <button onClick={() => onScheduleReport(report.id)}>Schedule</button>
              )}
              {onExportReport && (
                <button onClick={() => onExportReport(report.id)}>Export</button>
              )}
              {onDeleteReport && (
                <button onClick={() => onDeleteReport(report.id)} style={{ color: '#ef4444' }}>
                  Delete
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    );
  };

  // Render report list item
  const renderReportListItem = (report: ReportDefinition) => {
    const lastExecution = getLastExecution(report.id);
    const isSelected = selectedReports.has(report.id);

    return (
      <div
        key={report.id}
        style={{
          ...styles.reportListItem,
          ...(isSelected ? styles.reportListItemSelected : {}),
        }}
      >
        <div style={styles.reportListLeft}>
          <input
            type="checkbox"
            checked={isSelected}
            onChange={() => toggleReportSelection(report.id)}
            style={styles.checkbox}
          />

          <div style={styles.reportListInfo}>
            <div style={styles.reportListName}>{report.name}</div>
            <div style={styles.reportListMeta}>
              <span style={getStatusStyle(report.status)}>{report.status}</span>
              <span>•</span>
              <span>{report.type}</span>
              <span>•</span>
              <span>v{report.version}</span>
              {report.metadata.category && (
                <>
                  <span>•</span>
                  <span>{report.metadata.category}</span>
                </>
              )}
              <span>•</span>
              <span>Updated {new Date(report.metadata.updatedAt).toLocaleDateString()}</span>
            </div>
            {lastExecution && (
              <div style={styles.reportListExecution}>
                Last run: {new Date(lastExecution.startTime).toLocaleString()} -{' '}
                <span style={getExecutionStatusStyle(lastExecution.status)}>
                  {lastExecution.status}
                </span>
              </div>
            )}
          </div>
        </div>

        <div style={styles.reportListActions}>
          {onViewReport && (
            <button onClick={() => onViewReport(report.id)} style={styles.actionButton}>
              👁️
            </button>
          )}
          {onEditReport && (
            <button onClick={() => onEditReport(report)} style={styles.actionButton}>
              ✎
            </button>
          )}
          {onExecuteReport && (
            <button onClick={() => onExecuteReport(report.id)} style={styles.actionButton}>
              ▶
            </button>
          )}
        </div>
      </div>
    );
  };

  return (
    <div style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <h1 style={styles.title}>Reports</h1>
        <div style={styles.headerActions}>
          <div style={styles.viewModeToggle}>
            <button
              onClick={() => setViewMode('grid')}
              style={{
                ...styles.viewModeButton,
                ...(viewMode === 'grid' ? styles.viewModeButtonActive : {}),
              }}
            >
              ⊞
            </button>
            <button
              onClick={() => setViewMode('list')}
              style={{
                ...styles.viewModeButton,
                ...(viewMode === 'list' ? styles.viewModeButtonActive : {}),
              }}
            >
              ☰
            </button>
          </div>
          {onCreateReport && (
            <button onClick={onCreateReport} style={styles.createButton}>
              + New Report
            </button>
          )}
        </div>
      </div>

      {/* Filters */}
      <div style={styles.filters}>
        <input
          type="text"
          placeholder="Search reports..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          style={styles.searchInput}
        />

        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value as ReportStatus | 'all')}
          style={styles.filterSelect}
        >
          <option value="all">All Statuses</option>
          <option value="draft">Draft</option>
          <option value="published">Published</option>
          <option value="archived">Archived</option>
          <option value="scheduled">Scheduled</option>
        </select>

        <select
          value={categoryFilter}
          onChange={(e) => setCategoryFilter(e.target.value)}
          style={styles.filterSelect}
        >
          {categories.map((category) => (
            <option key={category} value={category}>
              {category === 'all' ? 'All Categories' : category}
            </option>
          ))}
        </select>

        <select
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as 'name' | 'updated' | 'created')}
          style={styles.filterSelect}
        >
          <option value="updated">Recently Updated</option>
          <option value="created">Recently Created</option>
          <option value="name">Name (A-Z)</option>
        </select>
      </div>

      {/* Bulk Actions */}
      {selectedReports.size > 0 && (
        <div style={styles.bulkActions}>
          <div style={styles.bulkActionsLeft}>
            <input
              type="checkbox"
              checked={selectedReports.size === filteredReports.length}
              onChange={toggleSelectAll}
              style={styles.checkbox}
            />
            <span style={styles.selectedCount}>{selectedReports.size} selected</span>
          </div>
          <div style={styles.bulkActionsRight}>
            {onDeleteReport && (
              <button onClick={handleBulkDelete} style={styles.bulkDeleteButton}>
                Delete Selected
              </button>
            )}
          </div>
        </div>
      )}

      {/* Content */}
      <div style={styles.content}>
        {filteredReports.length === 0 ? (
          <div style={styles.emptyState}>
            <div style={styles.emptyStateIcon}>📊</div>
            <div style={styles.emptyStateText}>No reports found</div>
            <div style={styles.emptyStateHint}>
              {reports.length === 0
                ? 'Create your first report to get started'
                : 'Try adjusting your search or filter criteria'}
            </div>
            {onCreateReport && reports.length === 0 && (
              <button onClick={onCreateReport} style={styles.emptyStateButton}>
                + Create Report
              </button>
            )}
          </div>
        ) : viewMode === 'grid' ? (
          <div style={styles.reportsGrid}>
            {filteredReports.map(renderReportCard)}
          </div>
        ) : (
          <div style={styles.reportsList}>
            {filteredReports.map(renderReportListItem)}
          </div>
        )}
      </div>

      {/* Stats Footer */}
      <div style={styles.footer}>
        <div style={styles.stats}>
          <div style={styles.statItem}>
            <span style={styles.statValue}>{reports.length}</span>
            <span style={styles.statLabel}>Total Reports</span>
          </div>
          <div style={styles.statItem}>
            <span style={styles.statValue}>
              {reports.filter((r) => r.status === 'published').length}
            </span>
            <span style={styles.statLabel}>Published</span>
          </div>
          <div style={styles.statItem}>
            <span style={styles.statValue}>
              {reports.filter((r) => r.schedule?.enabled).length}
            </span>
            <span style={styles.statLabel}>Scheduled</span>
          </div>
          <div style={styles.statItem}>
            <span style={styles.statValue}>{executions.length}</span>
            <span style={styles.statLabel}>Total Executions</span>
          </div>
        </div>
      </div>
    </div>
  );
};

// Helper functions
function getStatusStyle(status: ReportStatus): React.CSSProperties {
  const baseStyle: React.CSSProperties = {
    fontSize: '11px',
    padding: '2px 8px',
    borderRadius: '12px',
    fontWeight: 600,
  };

  const colors: Record<ReportStatus, { bg: string; color: string }> = {
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

function getExecutionStatusStyle(status: string): React.CSSProperties {
  const colors: Record<string, string> = {
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

// Styles
const styles: Record<string, React.CSSProperties> = {
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
