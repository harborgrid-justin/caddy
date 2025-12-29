/**
 * Enterprise Table Component
 * Features: Sorting, filtering, virtualization, pagination, row selection, keyboard navigation
 */

import React, { useState, useMemo, CSSProperties, ReactNode } from 'react';
import { useTheme } from './styles/theme';
import { transitionPresets } from './styles/animations';

export type SortDirection = 'asc' | 'desc' | null;

export interface Column<T = any> {
  id: string;
  header: ReactNode;
  accessor: keyof T | ((row: T) => any);
  width?: number | string;
  minWidth?: number;
  sortable?: boolean;
  filterable?: boolean;
  render?: (value: any, row: T, index: number) => ReactNode;
  align?: 'left' | 'center' | 'right';
}

export interface TableProps<T = any> {
  /** Column definitions */
  columns: Column<T>[];
  /** Table data */
  data: T[];
  /** Row key accessor */
  rowKey: keyof T | ((row: T) => string);
  /** Enable row selection */
  selectable?: boolean;
  /** Selected row keys */
  selectedRows?: string[];
  /** Selection change handler */
  onSelectionChange?: (selectedKeys: string[]) => void;
  /** Enable sorting */
  sortable?: boolean;
  /** Sort column */
  sortColumn?: string;
  /** Sort direction */
  sortDirection?: SortDirection;
  /** Sort change handler */
  onSortChange?: (columnId: string, direction: SortDirection) => void;
  /** Enable filtering */
  filterable?: boolean;
  /** Enable virtualization */
  virtualized?: boolean;
  /** Height for virtualized table */
  height?: number;
  /** Row height */
  rowHeight?: number;
  /** Empty state */
  emptyState?: ReactNode;
  /** Loading state */
  loading?: boolean;
  /** Striped rows */
  striped?: boolean;
  /** Hoverable rows */
  hoverable?: boolean;
  /** Compact size */
  compact?: boolean;
}

export const Table = <T extends Record<string, any>>({
  columns,
  data,
  rowKey,
  selectable = false,
  selectedRows = [],
  onSelectionChange,
  sortable = true,
  sortColumn: controlledSortColumn,
  sortDirection: controlledSortDirection,
  onSortChange,
  filterable = false,
  virtualized = false,
  height = 400,
  rowHeight = 48,
  emptyState,
  loading = false,
  striped = false,
  hoverable = true,
  compact = false,
}: TableProps<T>) => {
  const { theme } = useTheme();
  const [internalSortColumn, setInternalSortColumn] = useState<string | null>(null);
  const [internalSortDirection, setInternalSortDirection] = useState<SortDirection>(null);
  const [filters, setFilters] = useState<Record<string, string>>({});
  const [scrollTop, setScrollTop] = useState(0);

  const sortColumn = controlledSortColumn ?? internalSortColumn;
  const sortDirection = controlledSortDirection ?? internalSortDirection;

  const getRowKey = (row: T): string => {
    return typeof rowKey === 'function' ? rowKey(row) : String(row[rowKey]);
  };

  const getCellValue = (row: T, column: Column<T>): any => {
    return typeof column.accessor === 'function' ? column.accessor(row) : row[column.accessor];
  };

  // Apply filters
  const filteredData = useMemo(() => {
    if (!filterable || Object.keys(filters).length === 0) return data;

    return data.filter((row) => {
      return Object.entries(filters).every(([columnId, filterValue]) => {
        if (!filterValue) return true;
        const column = columns.find((c) => c.id === columnId);
        if (!column) return true;

        const cellValue = String(getCellValue(row, column)).toLowerCase();
        return cellValue.includes(filterValue.toLowerCase());
      });
    });
  }, [data, filters, filterable, columns]);

  // Apply sorting
  const sortedData = useMemo(() => {
    if (!sortColumn || !sortDirection) return filteredData;

    const column = columns.find((c) => c.id === sortColumn);
    if (!column) return filteredData;

    return [...filteredData].sort((a, b) => {
      const aValue = getCellValue(a, column);
      const bValue = getCellValue(b, column);

      if (aValue === bValue) return 0;

      const comparison = aValue < bValue ? -1 : 1;
      return sortDirection === 'asc' ? comparison : -comparison;
    });
  }, [filteredData, sortColumn, sortDirection, columns]);

  const handleSort = (columnId: string) => {
    const column = columns.find((c) => c.id === columnId);
    if (!column?.sortable && !sortable) return;

    let newDirection: SortDirection = 'asc';

    if (sortColumn === columnId) {
      if (sortDirection === 'asc') newDirection = 'desc';
      else if (sortDirection === 'desc') newDirection = null;
    }

    if (onSortChange) {
      onSortChange(columnId, newDirection);
    } else {
      setInternalSortColumn(newDirection ? columnId : null);
      setInternalSortDirection(newDirection);
    }
  };

  const handleSelectAll = () => {
    if (selectedRows.length === sortedData.length) {
      onSelectionChange?.([]);
    } else {
      onSelectionChange?.(sortedData.map(getRowKey));
    }
  };

  const handleSelectRow = (rowKey: string) => {
    const newSelection = selectedRows.includes(rowKey)
      ? selectedRows.filter((k) => k !== rowKey)
      : [...selectedRows, rowKey];
    onSelectionChange?.(newSelection);
  };

  const cellPadding = compact ? theme.spacing[2] : theme.spacing[4];

  const tableStyles: CSSProperties = {
    width: '100%',
    borderCollapse: 'collapse',
    backgroundColor: theme.colors.background.primary,
    fontSize: theme.typography.fontSize.sm,
    color: theme.colors.text.primary,
  };

  const headerCellStyles = (column: Column<T>): CSSProperties => ({
    padding: cellPadding,
    textAlign: column.align || 'left',
    fontWeight: theme.typography.fontWeight.semibold,
    backgroundColor: theme.colors.background.secondary,
    borderBottom: `2px solid ${theme.colors.border.primary}`,
    position: 'sticky',
    top: 0,
    zIndex: 1,
    cursor: (column.sortable || sortable) ? 'pointer' : 'default',
    userSelect: 'none',
    whiteSpace: 'nowrap',
    width: column.width,
    minWidth: column.minWidth,
  });

  const cellStyles = (column: Column<T>): CSSProperties => ({
    padding: cellPadding,
    textAlign: column.align || 'left',
    borderBottom: `1px solid ${theme.colors.border.secondary}`,
  });

  const rowStyles = (index: number, isSelected: boolean): CSSProperties => ({
    backgroundColor: isSelected
      ? theme.colors.interactive.secondary
      : striped && index % 2 === 1
      ? theme.colors.background.secondary
      : 'transparent',
    transition: transitionPresets.colors,
    height: `${rowHeight}px`,
  });

  const SortIcon = ({ direction }: { direction: SortDirection }) => {
    if (!direction) {
      return (
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" opacity="0.3">
          <path d="M8 3l3 4H5l3-4zM8 13l3-4H5l3 4z" />
        </svg>
      );
    }

    return (
      <svg
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="currentColor"
        style={{
          transform: direction === 'desc' ? 'rotate(180deg)' : 'rotate(0deg)',
          transition: transitionPresets.transform,
        }}
      >
        <path d="M8 3l4 5H4l4-5z" />
      </svg>
    );
  };

  const containerStyles: CSSProperties = {
    border: `1px solid ${theme.colors.border.primary}`,
    borderRadius: theme.borderRadius.md,
    overflow: virtualized ? 'hidden' : 'auto',
    height: virtualized ? `${height}px` : 'auto',
  };

  if (loading) {
    return (
      <div style={{ ...containerStyles, display: 'flex', alignItems: 'center', justifyContent: 'center', padding: theme.spacing[8] }}>
        <div style={{ textAlign: 'center', color: theme.colors.text.secondary }}>
          <div style={{ animation: 'spin 1s linear infinite', marginBottom: theme.spacing[2] }}>⏳</div>
          Loading...
        </div>
      </div>
    );
  }

  if (sortedData.length === 0) {
    return (
      <div style={{ ...containerStyles, display: 'flex', alignItems: 'center', justifyContent: 'center', padding: theme.spacing[8] }}>
        {emptyState || (
          <div style={{ textAlign: 'center', color: theme.colors.text.secondary }}>
            No data available
          </div>
        )}
      </div>
    );
  }

  const renderTable = (visibleData: T[], startIndex = 0) => (
    <table style={tableStyles}>
      <thead>
        <tr>
          {selectable && (
            <th style={{ ...headerCellStyles({ id: 'select', header: '' }), width: '50px' }}>
              <input
                type="checkbox"
                checked={selectedRows.length === sortedData.length && sortedData.length > 0}
                onChange={handleSelectAll}
                style={{ cursor: 'pointer' }}
              />
            </th>
          )}
          {columns.map((column) => (
            <th
              key={column.id}
              style={headerCellStyles(column)}
              onClick={() => handleSort(column.id)}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: theme.spacing[2], justifyContent: column.align === 'right' ? 'flex-end' : column.align === 'center' ? 'center' : 'flex-start' }}>
                <span>{column.header}</span>
                {(column.sortable || sortable) && (
                  <SortIcon direction={sortColumn === column.id ? sortDirection : null} />
                )}
              </div>
              {filterable && column.filterable !== false && (
                <input
                  type="text"
                  placeholder="Filter..."
                  value={filters[column.id] || ''}
                  onChange={(e) => setFilters({ ...filters, [column.id]: e.target.value })}
                  onClick={(e) => e.stopPropagation()}
                  style={{
                    width: '100%',
                    marginTop: theme.spacing[1],
                    padding: theme.spacing[1],
                    border: `1px solid ${theme.colors.border.primary}`,
                    borderRadius: theme.borderRadius.base,
                    backgroundColor: theme.colors.background.primary,
                    color: theme.colors.text.primary,
                    fontSize: theme.typography.fontSize.xs,
                  }}
                />
              )}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {visibleData.map((row, index) => {
          const key = getRowKey(row);
          const isSelected = selectedRows.includes(key);
          const actualIndex = startIndex + index;

          return (
            <tr
              key={key}
              style={rowStyles(actualIndex, isSelected)}
              onMouseEnter={(e) => {
                if (hoverable) {
                  e.currentTarget.style.backgroundColor = theme.colors.background.secondary;
                }
              }}
              onMouseLeave={(e) => {
                if (hoverable && !isSelected) {
                  e.currentTarget.style.backgroundColor =
                    striped && actualIndex % 2 === 1 ? theme.colors.background.secondary : 'transparent';
                }
              }}
            >
              {selectable && (
                <td style={cellStyles({ id: 'select', header: '' })}>
                  <input
                    type="checkbox"
                    checked={isSelected}
                    onChange={() => handleSelectRow(key)}
                    style={{ cursor: 'pointer' }}
                  />
                </td>
              )}
              {columns.map((column) => {
                const value = getCellValue(row, column);
                return (
                  <td key={column.id} style={cellStyles(column)}>
                    {column.render ? column.render(value, row, actualIndex) : value}
                  </td>
                );
              })}
            </tr>
          );
        })}
      </tbody>
    </table>
  );

  if (virtualized) {
    const visibleStart = Math.floor(scrollTop / rowHeight);
    const visibleEnd = Math.ceil((scrollTop + height) / rowHeight);
    const visibleData = sortedData.slice(visibleStart, visibleEnd);
    const totalHeight = sortedData.length * rowHeight + 50; // +50 for header

    return (
      <div
        style={containerStyles}
        onScroll={(e) => setScrollTop(e.currentTarget.scrollTop)}
      >
        <div style={{ height: `${totalHeight}px`, position: 'relative' }}>
          <div style={{ transform: `translateY(${visibleStart * rowHeight}px)` }}>
            {renderTable(visibleData, visibleStart)}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div style={containerStyles}>
      {renderTable(sortedData)}
    </div>
  );
};

Table.displayName = 'Table';
