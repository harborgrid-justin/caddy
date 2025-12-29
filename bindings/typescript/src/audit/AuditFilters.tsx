/**
 * Audit Filters Component
 * Advanced filtering interface for audit logs
 */

import React, { useState, useEffect } from 'react';
import type {
  AuditFilter,
  AuditEventType,
  AuditSeverity,
  AuditStatus,
  ResourceType,
} from './types';

interface AuditFiltersProps {
  filters: AuditFilter;
  onChange: (filters: AuditFilter) => void;
  onReset: () => void;
}

export const AuditFilters: React.FC<AuditFiltersProps> = ({
  filters,
  onChange,
  onReset,
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [localFilters, setLocalFilters] = useState<AuditFilter>(filters);

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

  const updateFilter = <K extends keyof AuditFilter>(
    key: K,
    value: AuditFilter[K]
  ) => {
    setLocalFilters((prev) => ({
      ...prev,
      [key]: value,
    }));
  };

  const toggleEventType = (eventType: AuditEventType) => {
    const current = localFilters.event_types || [];
    const updated = current.includes(eventType)
      ? current.filter((t) => t !== eventType)
      : [...current, eventType];
    updateFilter('event_types', updated.length > 0 ? updated : undefined);
  };

  const toggleSeverity = (severity: AuditSeverity) => {
    const current = localFilters.severities || [];
    const updated = current.includes(severity)
      ? current.filter((s) => s !== severity)
      : [...current, severity];
    updateFilter('severities', updated.length > 0 ? updated : undefined);
  };

  const toggleStatus = (status: AuditStatus) => {
    const current = localFilters.statuses || [];
    const updated = current.includes(status)
      ? current.filter((s) => s !== status)
      : [...current, status];
    updateFilter('statuses', updated.length > 0 ? updated : undefined);
  };

  const toggleResourceType = (resourceType: ResourceType) => {
    const current = localFilters.resource_types || [];
    const updated = current.includes(resourceType)
      ? current.filter((r) => r !== resourceType)
      : [...current, resourceType];
    updateFilter('resource_types', updated.length > 0 ? updated : undefined);
  };

  const activeFilterCount = Object.values(localFilters).filter(
    (v) => v !== undefined && (Array.isArray(v) ? v.length > 0 : true)
  ).length;

  return (
    <div className="audit-filters">
      {/* Quick Filters */}
      <div className="quick-filters">
        <div className="search-box">
          <input
            type="text"
            placeholder="Search events, users, resources..."
            value={localFilters.search_query || ''}
            onChange={(e) => updateFilter('search_query', e.target.value || undefined)}
            className="search-input"
          />
          <span className="search-icon">🔍</span>
        </div>

        <div className="quick-filter-buttons">
          <button
            className={`filter-chip ${localFilters.anomaly_only ? 'active' : ''}`}
            onClick={() => updateFilter('anomaly_only', !localFilters.anomaly_only)}
          >
            Anomalies Only
          </button>

          <button
            className={`filter-chip ${
              localFilters.min_risk_score !== undefined ? 'active' : ''
            }`}
            onClick={() =>
              updateFilter(
                'min_risk_score',
                localFilters.min_risk_score !== undefined ? undefined : 70
              )
            }
          >
            High Risk
          </button>

          <button
            className="filter-toggle"
            onClick={() => setIsExpanded(!isExpanded)}
          >
            Advanced Filters
            {activeFilterCount > 0 && (
              <span className="filter-count">{activeFilterCount}</span>
            )}
            <span className={`toggle-icon ${isExpanded ? 'expanded' : ''}`}>
              ▼
            </span>
          </button>
        </div>

        <div className="filter-actions">
          <button className="btn btn-secondary" onClick={handleResetFilters}>
            Reset
          </button>
          <button className="btn btn-primary" onClick={handleApplyFilters}>
            Apply Filters
          </button>
        </div>
      </div>

      {/* Advanced Filters */}
      {isExpanded && (
        <div className="advanced-filters">
          {/* Date Range */}
          <div className="filter-section">
            <h4>Date Range</h4>
            <div className="date-range">
              <div className="date-input-group">
                <label>Start Date</label>
                <input
                  type="datetime-local"
                  value={
                    localFilters.start_date
                      ? formatDateForInput(localFilters.start_date)
                      : ''
                  }
                  onChange={(e) =>
                    updateFilter(
                      'start_date',
                      e.target.value ? new Date(e.target.value) : undefined
                    )
                  }
                />
              </div>
              <div className="date-input-group">
                <label>End Date</label>
                <input
                  type="datetime-local"
                  value={
                    localFilters.end_date
                      ? formatDateForInput(localFilters.end_date)
                      : ''
                  }
                  onChange={(e) =>
                    updateFilter(
                      'end_date',
                      e.target.value ? new Date(e.target.value) : undefined
                    )
                  }
                />
              </div>
            </div>
          </div>

          {/* Event Types */}
          <div className="filter-section">
            <h4>Event Types</h4>
            <div className="filter-chips">
              {EVENT_TYPE_GROUPS.map((group) => (
                <div key={group.category} className="filter-group">
                  <h5>{group.category}</h5>
                  <div className="chip-grid">
                    {group.types.map((type) => (
                      <button
                        key={type}
                        className={`filter-chip ${
                          localFilters.event_types?.includes(type) ? 'active' : ''
                        }`}
                        onClick={() => toggleEventType(type)}
                      >
                        {formatEventType(type)}
                      </button>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Severities */}
          <div className="filter-section">
            <h4>Severity</h4>
            <div className="severity-filters">
              {(['low', 'medium', 'high', 'critical'] as AuditSeverity[]).map(
                (severity) => (
                  <button
                    key={severity}
                    className={`severity-chip severity-${severity} ${
                      localFilters.severities?.includes(severity) ? 'active' : ''
                    }`}
                    onClick={() => toggleSeverity(severity)}
                  >
                    {severity.toUpperCase()}
                  </button>
                )
              )}
            </div>
          </div>

          {/* Status */}
          <div className="filter-section">
            <h4>Status</h4>
            <div className="status-filters">
              {(['success', 'failure', 'pending', 'blocked'] as AuditStatus[]).map(
                (status) => (
                  <button
                    key={status}
                    className={`status-chip status-${status} ${
                      localFilters.statuses?.includes(status) ? 'active' : ''
                    }`}
                    onClick={() => toggleStatus(status)}
                  >
                    {status.charAt(0).toUpperCase() + status.slice(1)}
                  </button>
                )
              )}
            </div>
          </div>

          {/* Resource Types */}
          <div className="filter-section">
            <h4>Resource Types</h4>
            <div className="resource-filters">
              {RESOURCE_TYPES.map((resource) => (
                <button
                  key={resource.value}
                  className={`filter-chip ${
                    localFilters.resource_types?.includes(resource.value)
                      ? 'active'
                      : ''
                  }`}
                  onClick={() => toggleResourceType(resource.value)}
                >
                  {resource.label}
                </button>
              ))}
            </div>
          </div>

          {/* User & Session */}
          <div className="filter-section">
            <h4>User & Session</h4>
            <div className="input-grid">
              <div className="input-group">
                <label>User ID or Email</label>
                <input
                  type="text"
                  placeholder="Enter user ID or email"
                  value={localFilters.user_ids?.join(', ') || ''}
                  onChange={(e) =>
                    updateFilter(
                      'user_ids',
                      e.target.value
                        ? e.target.value.split(',').map((s) => s.trim())
                        : undefined
                    )
                  }
                />
              </div>
              <div className="input-group">
                <label>IP Address</label>
                <input
                  type="text"
                  placeholder="Enter IP address"
                  value={localFilters.ip_address || ''}
                  onChange={(e) =>
                    updateFilter('ip_address', e.target.value || undefined)
                  }
                />
              </div>
              <div className="input-group">
                <label>Session ID</label>
                <input
                  type="text"
                  placeholder="Enter session ID"
                  value={localFilters.session_id || ''}
                  onChange={(e) =>
                    updateFilter('session_id', e.target.value || undefined)
                  }
                />
              </div>
            </div>
          </div>

          {/* Risk Score */}
          <div className="filter-section">
            <h4>Risk Score</h4>
            <div className="risk-score-filter">
              <label>Minimum Risk Score: {localFilters.min_risk_score || 0}</label>
              <input
                type="range"
                min="0"
                max="100"
                step="10"
                value={localFilters.min_risk_score || 0}
                onChange={(e) =>
                  updateFilter(
                    'min_risk_score',
                    parseInt(e.target.value) || undefined
                  )
                }
                className="risk-slider"
              />
              <div className="risk-scale">
                <span>0</span>
                <span>25</span>
                <span>50</span>
                <span>75</span>
                <span>100</span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

// Event Type Groups
const EVENT_TYPE_GROUPS: {
  category: string;
  types: AuditEventType[];
}[] = [
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

// Resource Types
const RESOURCE_TYPES: { value: ResourceType; label: string }[] = [
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

// Utility Functions
function formatEventType(eventType: string): string {
  const parts = eventType.split('.');
  return parts[parts.length - 1]
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

function formatDateForInput(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  return `${year}-${month}-${day}T${hours}:${minutes}`;
}
