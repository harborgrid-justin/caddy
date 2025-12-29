/**
 * CADDY v0.4.0 - Report Viewer Component
 * $650M Platform - Production Ready
 *
 * Interactive report viewer with drill-down capabilities, parameter controls,
 * pagination, sorting, and real-time data refresh.
 */

import React, { useState, useEffect, useCallback, useMemo } from 'react';
import {
  ReportDefinition,
  ReportData,
  ReportParameter,
  ReportExecution,
  DrillDownConfig,
  Filter,
  OrderBy,
} from './types';

export interface ReportViewerProps {
  reportId: string;
  definition?: ReportDefinition;
  initialParameters?: Record<string, any>;
  onExecute?: (reportId: string, parameters: Record<string, any>) => Promise<ReportData>;
  onDrillDown?: (targetReportId: string, filters: Filter[]) => void;
  onExport?: (format: string) => void;
  autoRefresh?: boolean;
  refreshInterval?: number;
  showToolbar?: boolean;
  showParameters?: boolean;
  interactive?: boolean;
}

export const ReportViewer: React.FC<ReportViewerProps> = ({
  reportId,
  definition,
  initialParameters = {},
  onExecute,
  onDrillDown,
  onExport,
  autoRefresh = false,
  refreshInterval = 60000,
  showToolbar = true,
  showParameters = true,
  interactive = true,
}) => {
  const [loading, setLoading] = useState(false);
  const [data, setData] = useState<ReportData | null>(null);
  const [parameters, setParameters] = useState<Record<string, any>>(initialParameters);
  const [execution, setExecution] = useState<ReportExecution | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(50);
  const [sortConfig, setSortConfig] = useState<OrderBy[]>([]);
  const [drillPath, setDrillPath] = useState<DrillPathItem[]>([]);
  const [expandedRows, setExpandedRows] = useState<Set<number>>(new Set());

  interface DrillPathItem {
    reportId: string;
    reportName: string;
    filters: Filter[];
  }

  // Execute report
  const executeReport = useCallback(
    async (params: Record<string, any> = parameters) => {
      if (!onExecute) return;

      setLoading(true);
      setError(null);

      const execStart: ReportExecution = {
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
      } catch (err) {
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
      } finally {
        setLoading(false);
      }
    },
    [reportId, parameters, onExecute]
  );

  // Initial execution
  useEffect(() => {
    executeReport();
  }, []);

  // Auto-refresh
  useEffect(() => {
    if (!autoRefresh || !refreshInterval) return;

    const interval = setInterval(() => {
      executeReport();
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [autoRefresh, refreshInterval, executeReport]);

  // Parameter change handler
  const handleParameterChange = useCallback((name: string, value: any) => {
    setParameters((prev) => ({ ...prev, [name]: value }));
  }, []);

  // Run report with updated parameters
  const handleRunReport = useCallback(() => {
    setCurrentPage(1);
    executeReport(parameters);
  }, [parameters, executeReport]);

  // Sort handler
  const handleSort = useCallback((field: string) => {
    setSortConfig((prev) => {
      const existing = prev.find((s) => s.field === field);
      if (existing) {
        if (existing.direction === 'asc') {
          return prev.map((s) => (s.field === field ? { ...s, direction: 'desc' as const } : s));
        } else {
          return prev.filter((s) => s.field !== field);
        }
      } else {
        return [...prev, { field, direction: 'asc' as const }];
      }
    });
  }, []);

  // Drill-down handler
  const handleDrillDown = useCallback(
    (rowData: any, config?: DrillDownConfig) => {
      if (!config || !config.enabled || !onDrillDown) return;

      const level = config.levels[0];
      if (!level) return;

      const filters: Filter[] = level.filters || [];

      // Add context filters from current row
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
    },
    [reportId, definition, onDrillDown]
  );

  // Navigate back in drill path
  const handleDrillBack = useCallback(
    (index: number) => {
      const pathItem = drillPath[index];
      if (pathItem && onDrillDown) {
        onDrillDown(pathItem.reportId, pathItem.filters);
        setDrillPath((prev) => prev.slice(0, index));
      }
    },
    [drillPath, onDrillDown]
  );

  // Toggle row expansion
  const toggleRowExpansion = useCallback((rowIndex: number) => {
    setExpandedRows((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(rowIndex)) {
        newSet.delete(rowIndex);
      } else {
        newSet.add(rowIndex);
      }
      return newSet;
    });
  }, []);

  // Paginated and sorted data
  const processedData = useMemo(() => {
    if (!data) return null;

    let rows = [...data.rows];

    // Apply sorting
    if (sortConfig.length > 0) {
      rows.sort((a, b) => {
        for (const sort of sortConfig) {
          const colIndex = data.columns.findIndex((c) => c.name === sort.field);
          if (colIndex === -1) continue;

          const aVal = a[colIndex];
          const bVal = b[colIndex];

          if (aVal === bVal) continue;

          const comparison = aVal < bVal ? -1 : 1;
          return sort.direction === 'asc' ? comparison : -comparison;
        }
        return 0;
      });
    }

    // Apply pagination
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

  // Render parameter controls
  const renderParameterControls = () => {
    if (!definition?.parameters || definition.parameters.length === 0) return null;

    return (
      <div style={styles.parametersPanel}>
        <h3 style={styles.parametersPanelTitle}>Parameters</h3>
        <div style={styles.parametersGrid}>
          {definition.parameters.map((param) => (
            <div key={param.name} style={styles.parameterControl}>
              <label style={styles.parameterLabel}>
                {param.displayName}
                {param.required && <span style={styles.required}>*</span>}
              </label>
              {renderParameterInput(param, parameters[param.name], handleParameterChange)}
              {param.ui?.helpText && (
                <span style={styles.helpText}>{param.ui.helpText}</span>
              )}
            </div>
          ))}
        </div>
        <button onClick={handleRunReport} style={styles.runButton} disabled={loading}>
          {loading ? 'Running...' : '▶ Run Report'}
        </button>
      </div>
    );
  };

  // Render drill-down breadcrumb
  const renderDrillPath = () => {
    if (drillPath.length === 0) return null;

    return (
      <div style={styles.drillPath}>
        <button onClick={() => handleDrillBack(-1)} style={styles.drillBackButton}>
          ← Back to {definition?.name}
        </button>
        <div style={styles.breadcrumb}>
          {drillPath.map((item, index) => (
            <React.Fragment key={index}>
              <button
                onClick={() => handleDrillBack(index)}
                style={styles.breadcrumbItem}
              >
                {item.reportName}
              </button>
              <span style={styles.breadcrumbSeparator}>/</span>
            </React.Fragment>
          ))}
          <span style={styles.breadcrumbCurrent}>Current</span>
        </div>
      </div>
    );
  };

  // Render data table
  const renderTable = () => {
    if (!processedData) return null;

    return (
      <div style={styles.tableContainer}>
        <table style={styles.table}>
          <thead>
            <tr>
              {processedData.columns.map((column) => (
                <th
                  key={column.name}
                  style={styles.tableHeader}
                  onClick={() => interactive && handleSort(column.name)}
                >
                  <div style={styles.headerContent}>
                    <span>{column.displayName}</span>
                    {interactive && (
                      <span style={styles.sortIcon}>
                        {getSortIcon(column.name, sortConfig)}
                      </span>
                    )}
                  </div>
                </th>
              ))}
              {interactive && <th style={styles.tableHeader}>Actions</th>}
            </tr>
          </thead>
          <tbody>
            {processedData.rows.map((row, rowIndex) => (
              <React.Fragment key={rowIndex}>
                <tr
                  style={{
                    ...styles.tableRow,
                    backgroundColor: rowIndex % 2 === 0 ? '#ffffff' : '#f8fafc',
                  }}
                >
                  {row.map((cell, cellIndex) => (
                    <td key={cellIndex} style={styles.tableCell}>
                      {formatCellValue(cell, processedData.columns[cellIndex])}
                    </td>
                  ))}
                  {interactive && (
                    <td style={styles.tableCell}>
                      <button
                        onClick={() => toggleRowExpansion(rowIndex)}
                        style={styles.actionButton}
                        title="Expand/Collapse"
                      >
                        {expandedRows.has(rowIndex) ? '−' : '+'}
                      </button>
                      {definition?.layout.sections.some((s) => s.config?.drillDown?.enabled) && (
                        <button
                          onClick={() => {
                            const rowData = processedData.columns.reduce(
                              (acc, col, idx) => ({ ...acc, [col.name]: row[idx] }),
                              {}
                            );
                            handleDrillDown(
                              rowData,
                              definition.layout.sections.find((s) => s.config?.drillDown)
                                ?.config?.drillDown
                            );
                          }}
                          style={styles.actionButton}
                          title="Drill Down"
                        >
                          🔍
                        </button>
                      )}
                    </td>
                  )}
                </tr>
                {expandedRows.has(rowIndex) && (
                  <tr>
                    <td colSpan={processedData.columns.length + (interactive ? 1 : 0)}>
                      <div style={styles.expandedContent}>
                        <pre>{JSON.stringify(row, null, 2)}</pre>
                      </div>
                    </td>
                  </tr>
                )}
              </React.Fragment>
            ))}
          </tbody>
        </table>
      </div>
    );
  };

  // Render pagination
  const renderPagination = () => {
    if (!processedData || processedData.totalPages <= 1) return null;

    return (
      <div style={styles.pagination}>
        <div style={styles.paginationInfo}>
          Showing {((currentPage - 1) * pageSize) + 1} to{' '}
          {Math.min(currentPage * pageSize, data!.totalRows)} of {data!.totalRows} rows
        </div>
        <div style={styles.paginationControls}>
          <button
            onClick={() => setCurrentPage(1)}
            disabled={currentPage === 1}
            style={styles.paginationButton}
          >
            «
          </button>
          <button
            onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
            disabled={currentPage === 1}
            style={styles.paginationButton}
          >
            ‹
          </button>
          <span style={styles.pageNumber}>
            Page {currentPage} of {processedData.totalPages}
          </span>
          <button
            onClick={() => setCurrentPage((p) => Math.min(processedData.totalPages, p + 1))}
            disabled={currentPage === processedData.totalPages}
            style={styles.paginationButton}
          >
            ›
          </button>
          <button
            onClick={() => setCurrentPage(processedData.totalPages)}
            disabled={currentPage === processedData.totalPages}
            style={styles.paginationButton}
          >
            »
          </button>
          <select
            value={pageSize}
            onChange={(e) => {
              setPageSize(Number(e.target.value));
              setCurrentPage(1);
            }}
            style={styles.pageSizeSelect}
          >
            <option value={25}>25 rows</option>
            <option value={50}>50 rows</option>
            <option value={100}>100 rows</option>
            <option value={250}>250 rows</option>
          </select>
        </div>
      </div>
    );
  };

  return (
    <div style={styles.container}>
      {/* Toolbar */}
      {showToolbar && (
        <div style={styles.toolbar}>
          <div style={styles.toolbarLeft}>
            <h2 style={styles.reportTitle}>{definition?.name || 'Report'}</h2>
            {execution && (
              <span style={styles.executionBadge}>
                {execution.status === 'completed' && `✓ ${execution.duration}ms`}
                {execution.status === 'running' && '⟳ Running...'}
                {execution.status === 'failed' && '✗ Failed'}
              </span>
            )}
          </div>
          <div style={styles.toolbarRight}>
            <button onClick={() => executeReport()} style={styles.toolbarButton} disabled={loading}>
              🔄 Refresh
            </button>
            {onExport && (
              <div style={styles.exportDropdown}>
                <button style={styles.toolbarButton}>⬇ Export</button>
                <div style={styles.exportMenu}>
                  <button onClick={() => onExport('pdf')}>PDF</button>
                  <button onClick={() => onExport('excel')}>Excel</button>
                  <button onClick={() => onExport('csv')}>CSV</button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Parameters */}
      {showParameters && renderParameterControls()}

      {/* Drill Path */}
      {renderDrillPath()}

      {/* Content */}
      <div style={styles.content}>
        {loading && (
          <div style={styles.loadingOverlay}>
            <div style={styles.spinner}>Loading...</div>
          </div>
        )}

        {error && (
          <div style={styles.errorContainer}>
            <div style={styles.errorIcon}>⚠️</div>
            <div style={styles.errorMessage}>{error}</div>
            <button onClick={() => executeReport()} style={styles.retryButton}>
              Retry
            </button>
          </div>
        )}

        {!loading && !error && data && (
          <>
            {renderTable()}
            {renderPagination()}
          </>
        )}
      </div>
    </div>
  );
};

// Helper functions
function generateExecutionId(): string {
  return `exec-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

function renderParameterInput(
  param: ReportParameter,
  value: any,
  onChange: (name: string, value: any) => void
): React.ReactNode {
  switch (param.ui?.inputType || 'text') {
    case 'select':
      return (
        <select
          value={value || param.defaultValue || ''}
          onChange={(e) => onChange(param.name, e.target.value)}
          style={styles.parameterInput}
        >
          <option value="">Select...</option>
          {param.allowedValues?.map((val) => (
            <option key={String(val)} value={String(val)}>
              {String(val)}
            </option>
          ))}
        </select>
      );
    case 'date':
      return (
        <input
          type="date"
          value={value || param.defaultValue || ''}
          onChange={(e) => onChange(param.name, e.target.value)}
          style={styles.parameterInput}
        />
      );
    case 'number':
      return (
        <input
          type="number"
          value={value || param.defaultValue || ''}
          onChange={(e) => onChange(param.name, Number(e.target.value))}
          style={styles.parameterInput}
        />
      );
    case 'checkbox':
      return (
        <input
          type="checkbox"
          checked={value || param.defaultValue || false}
          onChange={(e) => onChange(param.name, e.target.checked)}
          style={styles.parameterCheckbox}
        />
      );
    default:
      return (
        <input
          type="text"
          value={value || param.defaultValue || ''}
          onChange={(e) => onChange(param.name, e.target.value)}
          placeholder={param.ui?.placeholder}
          style={styles.parameterInput}
        />
      );
  }
}

function getSortIcon(field: string, sortConfig: OrderBy[]): string {
  const sort = sortConfig.find((s) => s.field === field);
  if (!sort) return '↕';
  return sort.direction === 'asc' ? '↑' : '↓';
}

function formatCellValue(value: any, column: any): string {
  if (value === null || value === undefined) return '';

  if (column.format) {
    // Apply formatting based on format string
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

// Styles
const styles: Record<string, React.CSSProperties> = {
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
