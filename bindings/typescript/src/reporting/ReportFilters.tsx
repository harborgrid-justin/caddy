/**
 * CADDY v0.4.0 - Report Filters Component
 * $650M Platform - Production Ready
 *
 * Advanced dynamic filter builder with nested groups, complex conditions,
 * and intelligent field-type aware operators.
 */

import React, { useState, useCallback } from 'react';
import {
  Filter,
  FilterGroup,
  FilterOperator,
  DataType,
  Field,
  Table,
} from './types';

export interface ReportFiltersProps {
  availableTables: Table[];
  filterGroup: FilterGroup;
  onChange: (filterGroup: FilterGroup) => void;
  readOnly?: boolean;
}

export const ReportFilters: React.FC<ReportFiltersProps> = ({
  availableTables,
  filterGroup,
  onChange,
  readOnly = false,
}) => {
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set(['root']));

  // Get all available fields
  const allFields: Array<{ table: Table; field: Field }> = availableTables.flatMap((table) =>
    table.fields.map((field) => ({ table, field }))
  );

  // Add filter to group
  const addFilter = useCallback(
    (path: number[]) => {
      if (readOnly) return;

      const newFilter: Filter = {
        field: '',
        operator: 'eq',
        value: '',
      };

      const updated = updateFilterGroup(filterGroup, path, (group) => ({
        ...group,
        filters: [...group.filters, newFilter],
      }));

      onChange(updated);
    },
    [filterGroup, onChange, readOnly]
  );

  // Add filter group
  const addFilterGroup = useCallback(
    (path: number[]) => {
      if (readOnly) return;

      const newGroup: FilterGroup = {
        operator: 'and',
        filters: [],
      };

      const updated = updateFilterGroup(filterGroup, path, (group) => ({
        ...group,
        filters: [...group.filters, newGroup],
      }));

      onChange(updated);
    },
    [filterGroup, onChange, readOnly]
  );

  // Update filter
  const updateFilter = useCallback(
    (path: number[], updates: Partial<Filter | FilterGroup>) => {
      if (readOnly) return;

      const updated = updateFilterAtPath(filterGroup, path, updates);
      onChange(updated);
    },
    [filterGroup, onChange, readOnly]
  );

  // Remove filter
  const removeFilter = useCallback(
    (path: number[]) => {
      if (readOnly) return;

      const parentPath = path.slice(0, -1);
      const index = path[path.length - 1];

      const updated = updateFilterGroup(filterGroup, parentPath, (group) => ({
        ...group,
        filters: group.filters.filter((_, i) => i !== index),
      }));

      onChange(updated);
    },
    [filterGroup, onChange, readOnly]
  );

  // Toggle group expansion
  const toggleGroup = useCallback((groupId: string) => {
    setExpandedGroups((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(groupId)) {
        newSet.delete(groupId);
      } else {
        newSet.add(groupId);
      }
      return newSet;
    });
  }, []);

  // Render filter group
  const renderFilterGroup = (
    group: FilterGroup,
    path: number[] = [],
    groupId: string = 'root'
  ): React.ReactNode => {
    const isExpanded = expandedGroups.has(groupId);
    const isRoot = path.length === 0;

    return (
      <div
        style={{
          ...styles.filterGroup,
          ...(isRoot ? styles.filterGroupRoot : {}),
        }}
      >
        <div style={styles.filterGroupHeader}>
          <button
            onClick={() => toggleGroup(groupId)}
            style={styles.expandButton}
          >
            {isExpanded ? '▼' : '▶'}
          </button>

          <select
            value={group.operator}
            onChange={(e) =>
              updateFilter(path, { operator: e.target.value as 'and' | 'or' })
            }
            disabled={readOnly}
            style={styles.operatorSelect}
          >
            <option value="and">AND</option>
            <option value="or">OR</option>
          </select>

          <span style={styles.filterCount}>
            {group.filters.length} {group.filters.length === 1 ? 'condition' : 'conditions'}
          </span>

          {!readOnly && !isRoot && (
            <button
              onClick={() => removeFilter(path)}
              style={styles.removeGroupButton}
              title="Remove group"
            >
              ✕
            </button>
          )}
        </div>

        {isExpanded && (
          <div style={styles.filterGroupContent}>
            {group.filters.map((filter, index) => {
              const filterPath = [...path, index];
              const filterId = `${groupId}-${index}`;

              if (isFilterGroup(filter)) {
                return (
                  <div key={index}>
                    {renderFilterGroup(filter, filterPath, filterId)}
                  </div>
                );
              } else {
                return (
                  <div key={index}>
                    {renderFilter(filter, filterPath)}
                  </div>
                );
              }
            })}

            {!readOnly && (
              <div style={styles.addButtons}>
                <button
                  onClick={() => addFilter(path)}
                  style={styles.addButton}
                >
                  + Add Condition
                </button>
                <button
                  onClick={() => addFilterGroup(path)}
                  style={styles.addGroupButton}
                >
                  + Add Group
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    );
  };

  // Render individual filter
  const renderFilter = (filter: Filter, path: number[]): React.ReactNode => {
    const selectedFieldInfo = allFields.find(
      (f) => `${f.table.name}.${f.field.name}` === filter.field
    );
    const fieldType = selectedFieldInfo?.field.dataType || 'string';
    const availableOperators = getOperatorsForType(fieldType);

    return (
      <div style={styles.filter}>
        <select
          value={filter.field}
          onChange={(e) => {
            const newFieldInfo = allFields.find(
              (f) => `${f.table.name}.${f.field.name}` === e.target.value
            );
            const newType = newFieldInfo?.field.dataType || 'string';
            const newOperators = getOperatorsForType(newType);
            updateFilter(path, {
              field: e.target.value,
              valueType: newType,
              operator: newOperators.includes(filter.operator)
                ? filter.operator
                : newOperators[0],
            });
          }}
          disabled={readOnly}
          style={styles.fieldSelect}
        >
          <option value="">Select field...</option>
          {availableTables.map((table) => (
            <optgroup key={table.name} label={table.displayName || table.name}>
              {table.fields.map((field) => (
                <option
                  key={field.name}
                  value={`${table.name}.${field.name}`}
                >
                  {field.displayName || field.name}
                </option>
              ))}
            </optgroup>
          ))}
        </select>

        <select
          value={filter.operator}
          onChange={(e) =>
            updateFilter(path, { operator: e.target.value as FilterOperator })
          }
          disabled={readOnly || !filter.field}
          style={styles.operatorSelectSmall}
        >
          {availableOperators.map((op) => (
            <option key={op.value} value={op.value}>
              {op.label}
            </option>
          ))}
        </select>

        {renderValueInput(filter, fieldType, path)}

        {!readOnly && (
          <button
            onClick={() => removeFilter(path)}
            style={styles.removeButton}
            title="Remove condition"
          >
            ✕
          </button>
        )}
      </div>
    );
  };

  // Render value input based on field type and operator
  const renderValueInput = (
    filter: Filter,
    fieldType: DataType,
    path: number[]
  ): React.ReactNode => {
    if (filter.operator === 'isNull' || filter.operator === 'isNotNull') {
      return <div style={styles.noValuePlaceholder}>No value needed</div>;
    }

    if (filter.operator === 'between') {
      return (
        <div style={styles.betweenInputs}>
          <input
            type={getInputType(fieldType)}
            value={Array.isArray(filter.value) ? filter.value[0] : ''}
            onChange={(e) =>
              updateFilter(path, {
                value: [e.target.value, Array.isArray(filter.value) ? filter.value[1] : ''],
              })
            }
            disabled={readOnly}
            style={styles.valueInput}
            placeholder="From"
          />
          <span style={styles.betweenSeparator}>and</span>
          <input
            type={getInputType(fieldType)}
            value={Array.isArray(filter.value) ? filter.value[1] : ''}
            onChange={(e) =>
              updateFilter(path, {
                value: [Array.isArray(filter.value) ? filter.value[0] : '', e.target.value],
              })
            }
            disabled={readOnly}
            style={styles.valueInput}
            placeholder="To"
          />
        </div>
      );
    }

    if (filter.operator === 'in' || filter.operator === 'nin') {
      return (
        <input
          type="text"
          value={Array.isArray(filter.value) ? filter.value.join(', ') : filter.value}
          onChange={(e) =>
            updateFilter(path, {
              value: e.target.value.split(',').map((v) => v.trim()),
            })
          }
          disabled={readOnly}
          style={styles.valueInput}
          placeholder="Comma-separated values"
        />
      );
    }

    if (fieldType === 'boolean') {
      return (
        <select
          value={String(filter.value)}
          onChange={(e) => updateFilter(path, { value: e.target.value === 'true' })}
          disabled={readOnly}
          style={styles.valueInput}
        >
          <option value="">Select...</option>
          <option value="true">True</option>
          <option value="false">False</option>
        </select>
      );
    }

    return (
      <input
        type={getInputType(fieldType)}
        value={String(filter.value || '')}
        onChange={(e) => {
          const value =
            fieldType === 'number' ? Number(e.target.value) : e.target.value;
          updateFilter(path, { value });
        }}
        disabled={readOnly}
        style={styles.valueInput}
        placeholder="Value"
      />
    );
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Filters</h3>
        {!readOnly && filterGroup.filters.length > 0 && (
          <button
            onClick={() => onChange({ operator: 'and', filters: [] })}
            style={styles.clearAllButton}
          >
            Clear All
          </button>
        )}
      </div>

      <div style={styles.content}>
        {renderFilterGroup(filterGroup)}

        {filterGroup.filters.length === 0 && (
          <div style={styles.emptyState}>
            <div style={styles.emptyStateIcon}>🔍</div>
            <div style={styles.emptyStateText}>No filters applied</div>
            {!readOnly && (
              <button
                onClick={() => addFilter([])}
                style={styles.addFirstButton}
              >
                + Add First Condition
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

// Helper functions
function isFilterGroup(filter: Filter | FilterGroup): filter is FilterGroup {
  return 'operator' in filter && 'filters' in filter && Array.isArray(filter.filters);
}

function updateFilterGroup(
  group: FilterGroup,
  path: number[],
  updater: (group: FilterGroup) => FilterGroup
): FilterGroup {
  if (path.length === 0) {
    return updater(group);
  }

  const [index, ...rest] = path;
  const target = group.filters[index];

  if (isFilterGroup(target)) {
    return {
      ...group,
      filters: group.filters.map((f, i) =>
        i === index ? updateFilterGroup(target, rest, updater) : f
      ),
    };
  }

  return group;
}

function updateFilterAtPath(
  group: FilterGroup,
  path: number[],
  updates: Partial<Filter | FilterGroup>
): FilterGroup {
  if (path.length === 0) {
    return { ...group, ...updates };
  }

  const [index, ...rest] = path;
  const target = group.filters[index];

  if (rest.length === 0) {
    return {
      ...group,
      filters: group.filters.map((f, i) =>
        i === index ? { ...f, ...updates } : f
      ),
    };
  }

  if (isFilterGroup(target)) {
    return {
      ...group,
      filters: group.filters.map((f, i) =>
        i === index ? updateFilterAtPath(target, rest, updates) : f
      ),
    };
  }

  return group;
}

function getOperatorsForType(
  dataType: DataType
): Array<{ value: FilterOperator; label: string }> {
  const commonOperators = [
    { value: 'eq' as FilterOperator, label: 'Equals' },
    { value: 'ne' as FilterOperator, label: 'Not Equals' },
    { value: 'isNull' as FilterOperator, label: 'Is Null' },
    { value: 'isNotNull' as FilterOperator, label: 'Is Not Null' },
  ];

  switch (dataType) {
    case 'number':
    case 'date':
      return [
        ...commonOperators,
        { value: 'gt', label: 'Greater Than' },
        { value: 'gte', label: 'Greater Than or Equal' },
        { value: 'lt', label: 'Less Than' },
        { value: 'lte', label: 'Less Than or Equal' },
        { value: 'between', label: 'Between' },
        { value: 'in', label: 'In' },
        { value: 'nin', label: 'Not In' },
      ];

    case 'string':
      return [
        ...commonOperators,
        { value: 'contains', label: 'Contains' },
        { value: 'startsWith', label: 'Starts With' },
        { value: 'endsWith', label: 'Ends With' },
        { value: 'in', label: 'In' },
        { value: 'nin', label: 'Not In' },
      ];

    case 'boolean':
      return commonOperators;

    default:
      return commonOperators;
  }
}

function getInputType(dataType: DataType): string {
  switch (dataType) {
    case 'number':
      return 'number';
    case 'date':
      return 'date';
    default:
      return 'text';
  }
}

// Styles
const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#ffffff',
    border: '1px solid #e2e8f0',
    borderRadius: '8px',
    fontFamily: 'Inter, system-ui, sans-serif',
    overflow: 'hidden',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '12px 16px',
    borderBottom: '1px solid #e2e8f0',
    backgroundColor: '#f8fafc',
  },
  title: {
    fontSize: '14px',
    fontWeight: 600,
    margin: 0,
    color: '#1e293b',
  },
  clearAllButton: {
    padding: '4px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    backgroundColor: '#ffffff',
    cursor: 'pointer',
    fontSize: '12px',
    color: '#ef4444',
  },
  content: {
    flex: 1,
    overflow: 'auto',
    padding: '16px',
  },
  filterGroup: {
    border: '1px solid #cbd5e1',
    borderRadius: '6px',
    marginBottom: '8px',
    backgroundColor: '#f8fafc',
  },
  filterGroupRoot: {
    border: '2px solid #2563eb',
    backgroundColor: '#ffffff',
  },
  filterGroupHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '8px 12px',
    backgroundColor: '#f1f5f9',
    borderBottom: '1px solid #e2e8f0',
  },
  expandButton: {
    border: 'none',
    background: 'none',
    cursor: 'pointer',
    fontSize: '12px',
    color: '#64748b',
    padding: '4px',
  },
  operatorSelect: {
    padding: '4px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: 600,
    backgroundColor: '#ffffff',
    cursor: 'pointer',
    color: '#2563eb',
  },
  filterCount: {
    fontSize: '11px',
    color: '#64748b',
    marginLeft: 'auto',
  },
  removeGroupButton: {
    border: 'none',
    background: 'none',
    color: '#ef4444',
    cursor: 'pointer',
    fontSize: '16px',
    padding: '0 4px',
  },
  filterGroupContent: {
    padding: '12px',
  },
  filter: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '8px',
    backgroundColor: '#ffffff',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    marginBottom: '8px',
  },
  fieldSelect: {
    flex: 1,
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '12px',
    cursor: 'pointer',
  },
  operatorSelectSmall: {
    width: '140px',
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '12px',
    cursor: 'pointer',
  },
  valueInput: {
    flex: 1,
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '12px',
  },
  noValuePlaceholder: {
    flex: 1,
    padding: '6px 8px',
    fontSize: '12px',
    color: '#94a3b8',
    fontStyle: 'italic',
  },
  betweenInputs: {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  betweenSeparator: {
    fontSize: '12px',
    color: '#64748b',
  },
  removeButton: {
    border: 'none',
    background: 'none',
    color: '#ef4444',
    cursor: 'pointer',
    fontSize: '16px',
    padding: '0 4px',
  },
  addButtons: {
    display: 'flex',
    gap: '8px',
    marginTop: '8px',
  },
  addButton: {
    flex: 1,
    padding: '6px 12px',
    border: '1px dashed #2563eb',
    borderRadius: '4px',
    backgroundColor: '#eff6ff',
    color: '#2563eb',
    cursor: 'pointer',
    fontSize: '12px',
    fontWeight: 500,
  },
  addGroupButton: {
    flex: 1,
    padding: '6px 12px',
    border: '1px dashed #64748b',
    borderRadius: '4px',
    backgroundColor: '#f8fafc',
    color: '#64748b',
    cursor: 'pointer',
    fontSize: '12px',
    fontWeight: 500,
  },
  emptyState: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '48px 16px',
    gap: '12px',
  },
  emptyStateIcon: {
    fontSize: '32px',
  },
  emptyStateText: {
    fontSize: '13px',
    color: '#64748b',
  },
  addFirstButton: {
    padding: '8px 16px',
    border: 'none',
    borderRadius: '6px',
    backgroundColor: '#2563eb',
    color: '#ffffff',
    cursor: 'pointer',
    fontSize: '13px',
    fontWeight: 500,
  },
};

export default ReportFilters;
