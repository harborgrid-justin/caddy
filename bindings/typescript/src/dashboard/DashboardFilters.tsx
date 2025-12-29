/**
 * CADDY Enterprise Dashboard Filters Component v0.4.0
 *
 * Advanced filtering system with date range, department, region, and custom filters.
 * Includes presets, saved filters, and accessibility support.
 */

import React, { useState, useEffect, useCallback, useMemo } from 'react';
import type { DashboardFilters, TimeRange } from './types';
import { useDashboard } from './DashboardLayout';

/**
 * Dashboard filters props
 */
export interface DashboardFiltersProps {
  /** Current filters */
  filters: DashboardFilters;
  /** On filters change */
  onChange: (filters: DashboardFilters) => void;
  /** Available departments */
  departments?: string[];
  /** Available regions */
  regions?: string[];
  /** Available users */
  users?: string[];
  /** Available statuses */
  statuses?: string[];
  /** Show date range picker */
  showDateRange?: boolean;
  /** Show department filter */
  showDepartments?: boolean;
  /** Show region filter */
  showRegions?: boolean;
  /** Show user filter */
  showUsers?: boolean;
  /** Show status filter */
  showStatuses?: boolean;
  /** Enable saved filters */
  enableSavedFilters?: boolean;
  /** Custom class name */
  className?: string;
}

/**
 * Dashboard filters component
 */
export const DashboardFiltersComponent: React.FC<DashboardFiltersProps> = ({
  filters,
  onChange,
  departments = [],
  regions = [],
  users = [],
  statuses = [],
  showDateRange = true,
  showDepartments = true,
  showRegions = true,
  showUsers = true,
  showStatuses = true,
  enableSavedFilters = true,
  className = '',
}) => {
  const [localFilters, setLocalFilters] = useState<DashboardFilters>(filters);
  const [isExpanded, setIsExpanded] = useState(false);
  const [savedFilters, setSavedFilters] = useState<SavedFilter[]>([]);
  const [filterName, setFilterName] = useState('');
  const { theme, accessibility } = useDashboard();

  /**
   * Time range presets
   */
  const timeRangePresets: { value: TimeRange; label: string }[] = [
    { value: '1h', label: 'Last Hour' },
    { value: '24h', label: 'Last 24 Hours' },
    { value: '7d', label: 'Last 7 Days' },
    { value: '30d', label: 'Last 30 Days' },
    { value: '90d', label: 'Last 90 Days' },
    { value: '1y', label: 'Last Year' },
    { value: 'custom', label: 'Custom Range' },
  ];

  /**
   * Load saved filters from localStorage
   */
  useEffect(() => {
    if (enableSavedFilters) {
      const saved = localStorage.getItem('dashboard-saved-filters');
      if (saved) {
        try {
          setSavedFilters(JSON.parse(saved));
        } catch (error) {
          console.error('Failed to load saved filters:', error);
        }
      }
    }
  }, [enableSavedFilters]);

  /**
   * Update local filters when prop changes
   */
  useEffect(() => {
    setLocalFilters(filters);
  }, [filters]);

  /**
   * Handle filter change
   */
  const handleFilterChange = useCallback(
    (updates: Partial<DashboardFilters>) => {
      const updated = { ...localFilters, ...updates };
      setLocalFilters(updated);
      onChange(updated);
    },
    [localFilters, onChange]
  );

  /**
   * Handle time range change
   */
  const handleTimeRangeChange = useCallback(
    (timeRange: TimeRange) => {
      handleFilterChange({
        timeRange,
        ...(timeRange !== 'custom' && { startDate: undefined, endDate: undefined }),
      });
    },
    [handleFilterChange]
  );

  /**
   * Handle custom date range
   */
  const handleCustomDateRange = useCallback(
    (startDate: string, endDate: string) => {
      handleFilterChange({
        timeRange: 'custom',
        startDate,
        endDate,
      });
    },
    [handleFilterChange]
  );

  /**
   * Toggle department selection
   */
  const toggleDepartment = useCallback(
    (department: string) => {
      const current = localFilters.departments || [];
      const updated = current.includes(department)
        ? current.filter((d) => d !== department)
        : [...current, department];
      handleFilterChange({ departments: updated });
    },
    [localFilters.departments, handleFilterChange]
  );

  /**
   * Toggle region selection
   */
  const toggleRegion = useCallback(
    (region: string) => {
      const current = localFilters.regions || [];
      const updated = current.includes(region)
        ? current.filter((r) => r !== region)
        : [...current, region];
      handleFilterChange({ regions: updated });
    },
    [localFilters.regions, handleFilterChange]
  );

  /**
   * Toggle user selection
   */
  const toggleUser = useCallback(
    (user: string) => {
      const current = localFilters.users || [];
      const updated = current.includes(user)
        ? current.filter((u) => u !== user)
        : [...current, user];
      handleFilterChange({ users: updated });
    },
    [localFilters.users, handleFilterChange]
  );

  /**
   * Toggle status selection
   */
  const toggleStatus = useCallback(
    (status: string) => {
      const current = localFilters.statuses || [];
      const updated = current.includes(status)
        ? current.filter((s) => s !== status)
        : [...current, status];
      handleFilterChange({ statuses: updated });
    },
    [localFilters.statuses, handleFilterChange]
  );

  /**
   * Reset all filters
   */
  const resetFilters = useCallback(() => {
    const reset: DashboardFilters = {
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

  /**
   * Save current filters
   */
  const saveFilters = useCallback(() => {
    if (!filterName) {
      alert('Please enter a name for this filter');
      return;
    }

    const newFilter: SavedFilter = {
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

  /**
   * Load saved filter
   */
  const loadSavedFilter = useCallback(
    (savedFilter: SavedFilter) => {
      setLocalFilters(savedFilter.filters);
      onChange(savedFilter.filters);
    },
    [onChange]
  );

  /**
   * Delete saved filter
   */
  const deleteSavedFilter = useCallback(
    (filterId: string) => {
      const updated = savedFilters.filter((f) => f.id !== filterId);
      setSavedFilters(updated);
      localStorage.setItem('dashboard-saved-filters', JSON.stringify(updated));
    },
    [savedFilters]
  );

  /**
   * Count active filters
   */
  const activeFilterCount = useMemo(() => {
    let count = 0;
    if (localFilters.departments?.length) count++;
    if (localFilters.regions?.length) count++;
    if (localFilters.users?.length) count++;
    if (localFilters.statuses?.length) count++;
    if (localFilters.custom && Object.keys(localFilters.custom).length) count++;
    return count;
  }, [localFilters]);

  return (
    <div
      className={`dashboard-filters ${className}`}
      style={styles.container}
      role="region"
      aria-label="Dashboard filters"
    >
      {/* Header */}
      <div style={styles.header}>
        <h3 style={styles.title}>
          Filters
          {activeFilterCount > 0 && (
            <span style={styles.activeCount} aria-label={`${activeFilterCount} active filters`}>
              {activeFilterCount}
            </span>
          )}
        </h3>

        <div style={styles.headerActions}>
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            style={styles.expandButton}
            aria-label={isExpanded ? 'Collapse filters' : 'Expand filters'}
            aria-expanded={isExpanded}
          >
            {isExpanded ? '▲' : '▼'}
          </button>

          {activeFilterCount > 0 && (
            <button
              onClick={resetFilters}
              style={styles.resetButton}
              aria-label="Reset all filters"
            >
              Reset
            </button>
          )}
        </div>
      </div>

      {/* Filters content */}
      {isExpanded && (
        <div style={styles.content}>
          {/* Time Range */}
          {showDateRange && (
            <div style={styles.filterGroup}>
              <label style={styles.label} id="time-range-label">
                Time Range
              </label>
              <select
                value={localFilters.timeRange}
                onChange={(e) => handleTimeRangeChange(e.target.value as TimeRange)}
                style={styles.select}
                aria-labelledby="time-range-label"
              >
                {timeRangePresets.map((preset) => (
                  <option key={preset.value} value={preset.value}>
                    {preset.label}
                  </option>
                ))}
              </select>

              {/* Custom date range */}
              {localFilters.timeRange === 'custom' && (
                <div style={styles.customDateRange}>
                  <input
                    type="datetime-local"
                    value={localFilters.startDate || ''}
                    onChange={(e) =>
                      handleCustomDateRange(e.target.value, localFilters.endDate || '')
                    }
                    style={styles.dateInput}
                    aria-label="Start date"
                  />
                  <span style={styles.dateSeparator}>to</span>
                  <input
                    type="datetime-local"
                    value={localFilters.endDate || ''}
                    onChange={(e) =>
                      handleCustomDateRange(localFilters.startDate || '', e.target.value)
                    }
                    style={styles.dateInput}
                    aria-label="End date"
                  />
                </div>
              )}
            </div>
          )}

          {/* Departments */}
          {showDepartments && departments.length > 0 && (
            <div style={styles.filterGroup}>
              <label style={styles.label} id="departments-label">
                Departments
              </label>
              <div style={styles.checkboxGroup} role="group" aria-labelledby="departments-label">
                {departments.map((dept) => (
                  <label key={dept} style={styles.checkboxLabel}>
                    <input
                      type="checkbox"
                      checked={localFilters.departments?.includes(dept) || false}
                      onChange={() => toggleDepartment(dept)}
                      style={styles.checkbox}
                      aria-label={dept}
                    />
                    <span style={styles.checkboxText}>{dept}</span>
                  </label>
                ))}
              </div>
            </div>
          )}

          {/* Regions */}
          {showRegions && regions.length > 0 && (
            <div style={styles.filterGroup}>
              <label style={styles.label} id="regions-label">
                Regions
              </label>
              <div style={styles.checkboxGroup} role="group" aria-labelledby="regions-label">
                {regions.map((region) => (
                  <label key={region} style={styles.checkboxLabel}>
                    <input
                      type="checkbox"
                      checked={localFilters.regions?.includes(region) || false}
                      onChange={() => toggleRegion(region)}
                      style={styles.checkbox}
                      aria-label={region}
                    />
                    <span style={styles.checkboxText}>{region}</span>
                  </label>
                ))}
              </div>
            </div>
          )}

          {/* Users */}
          {showUsers && users.length > 0 && (
            <div style={styles.filterGroup}>
              <label style={styles.label} id="users-label">
                Users
              </label>
              <div style={styles.checkboxGroup} role="group" aria-labelledby="users-label">
                {users.map((user) => (
                  <label key={user} style={styles.checkboxLabel}>
                    <input
                      type="checkbox"
                      checked={localFilters.users?.includes(user) || false}
                      onChange={() => toggleUser(user)}
                      style={styles.checkbox}
                      aria-label={user}
                    />
                    <span style={styles.checkboxText}>{user}</span>
                  </label>
                ))}
              </div>
            </div>
          )}

          {/* Statuses */}
          {showStatuses && statuses.length > 0 && (
            <div style={styles.filterGroup}>
              <label style={styles.label} id="statuses-label">
                Statuses
              </label>
              <div style={styles.checkboxGroup} role="group" aria-labelledby="statuses-label">
                {statuses.map((status) => (
                  <label key={status} style={styles.checkboxLabel}>
                    <input
                      type="checkbox"
                      checked={localFilters.statuses?.includes(status) || false}
                      onChange={() => toggleStatus(status)}
                      style={styles.checkbox}
                      aria-label={status}
                    />
                    <span style={styles.checkboxText}>{status}</span>
                  </label>
                ))}
              </div>
            </div>
          )}

          {/* Save Filters */}
          {enableSavedFilters && (
            <div style={styles.filterGroup}>
              <label style={styles.label} id="save-filter-label">
                Save Current Filters
              </label>
              <div style={styles.saveFilterRow}>
                <input
                  type="text"
                  value={filterName}
                  onChange={(e) => setFilterName(e.target.value)}
                  placeholder="Filter name..."
                  style={styles.filterNameInput}
                  aria-labelledby="save-filter-label"
                />
                <button
                  onClick={saveFilters}
                  style={styles.saveButton}
                  disabled={!filterName}
                  aria-label="Save filter"
                >
                  Save
                </button>
              </div>
            </div>
          )}

          {/* Saved Filters List */}
          {enableSavedFilters && savedFilters.length > 0 && (
            <div style={styles.filterGroup}>
              <label style={styles.label}>Saved Filters</label>
              <div style={styles.savedFiltersList}>
                {savedFilters.map((saved) => (
                  <div key={saved.id} style={styles.savedFilterItem}>
                    <button
                      onClick={() => loadSavedFilter(saved)}
                      style={styles.savedFilterButton}
                      aria-label={`Load filter: ${saved.name}`}
                    >
                      {saved.name}
                    </button>
                    <button
                      onClick={() => deleteSavedFilter(saved.id)}
                      style={styles.deleteFilterButton}
                      aria-label={`Delete filter: ${saved.name}`}
                    >
                      ×
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

/**
 * Saved filter interface
 */
interface SavedFilter {
  id: string;
  name: string;
  filters: DashboardFilters;
  createdAt: string;
}

/**
 * Component styles
 */
const styles: Record<string, React.CSSProperties> = {
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
