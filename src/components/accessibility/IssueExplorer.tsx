/**
 * Issue Explorer
 *
 * Advanced issue browsing component with filtering, sorting, highlighting,
 * and inline fix suggestions.
 */

import React, { useState, useMemo } from 'react';
import { useTheme } from '../enterprise/styles/theme';
import { Button } from '../enterprise/Button';
import { Table } from '../enterprise/Table';
import { Input } from '../enterprise/Input';
import { Select } from '../enterprise/Select';
import { Modal } from '../enterprise/Modal';
import {
  useAccessibility,
  useFilteredIssues,
  useBulkIssueOperations,
  useIssueHighlight,
  useDebouncedSearch,
} from './useAccessibility';
import {
  AccessibilityIssue,
  IssueLevel,
  IssueCategory,
  IssueFilter,
  IssueSortConfig,
} from './types';

interface IssueExplorerProps {
  initialFilter?: IssueFilter;
  onIssueClick?: (issue: AccessibilityIssue) => void;
  showBulkActions?: boolean;
}

export function IssueExplorer({
  initialFilter,
  onIssueClick,
  showBulkActions = true,
}: IssueExplorerProps) {
  const { theme } = useTheme();
  const { updateIssue, deleteIssue, markIssueAsFixed, markIssueAsFalsePositive } = useAccessibility();
  const { searchTerm, setSearchTerm, debouncedTerm } = useDebouncedSearch();

  // Filters
  const [selectedLevels, setSelectedLevels] = useState<IssueLevel[]>([]);
  const [selectedCategories, setSelectedCategories] = useState<IssueCategory[]>([]);
  const [selectedStatus, setSelectedStatus] = useState<string[]>([]);
  const [sortConfig, setSortConfig] = useState<IssueSortConfig>({
    field: 'detectedAt',
    direction: 'desc',
  });

  // Modal state
  const [selectedIssue, setSelectedIssue] = useState<AccessibilityIssue | null>(null);
  const [showDetailsModal, setShowDetailsModal] = useState(false);

  // Build filter object
  const filter: IssueFilter = useMemo(
    () => ({
      ...initialFilter,
      levels: selectedLevels.length > 0 ? selectedLevels : undefined,
      categories: selectedCategories.length > 0 ? selectedCategories : undefined,
      status: selectedStatus.length > 0 ? selectedStatus as any : undefined,
      searchQuery: debouncedTerm || undefined,
    }),
    [initialFilter, selectedLevels, selectedCategories, selectedStatus, debouncedTerm]
  );

  // Get filtered issues
  const filteredIssues = useFilteredIssues(filter, sortConfig);

  // Bulk operations
  const {
    selectedIssues,
    toggleIssue,
    selectAll,
    deselectAll,
    bulkUpdate,
  } = useBulkIssueOperations();

  // Issue highlighting
  const { highlightedIssueId, highlight, unhighlight } = useIssueHighlight();

  const handleIssueClick = (issue: AccessibilityIssue) => {
    setSelectedIssue(issue);
    setShowDetailsModal(true);
    highlight(issue);
    onIssueClick?.(issue);
  };

  const handleCloseDetails = () => {
    setShowDetailsModal(false);
    setSelectedIssue(null);
    unhighlight();
  };

  const handleMarkAsFixed = async (issueId: string) => {
    await markIssueAsFixed(issueId);
    if (selectedIssue?.id === issueId) {
      handleCloseDetails();
    }
  };

  const handleMarkAsFalsePositive = async (issueId: string, reason: string) => {
    await markIssueAsFalsePositive(issueId, reason);
    if (selectedIssue?.id === issueId) {
      handleCloseDetails();
    }
  };

  const handleDelete = async (issueId: string) => {
    if (confirm('Are you sure you want to delete this issue?')) {
      await deleteIssue(issueId);
      if (selectedIssue?.id === issueId) {
        handleCloseDetails();
      }
    }
  };

  const levelColors = {
    [IssueLevel.Critical]: '#dc2626',
    [IssueLevel.Serious]: '#ea580c',
    [IssueLevel.Moderate]: '#f59e0b',
    [IssueLevel.Minor]: '#10b981',
  };

  const columns = [
    {
      id: 'level',
      header: 'Severity',
      accessor: 'level' as keyof AccessibilityIssue,
      width: '120px',
      sortable: true,
      render: (value: IssueLevel) => (
        <div
          style={{
            display: 'inline-block',
            padding: `${theme.spacing[1]} ${theme.spacing[2]}`,
            backgroundColor: levelColors[value],
            color: '#ffffff',
            borderRadius: theme.borderRadius.base,
            fontSize: theme.typography.fontSize.xs,
            fontWeight: theme.typography.fontWeight.semibold,
            textTransform: 'uppercase',
          }}
        >
          {value}
        </div>
      ),
    },
    {
      id: 'title',
      header: 'Issue',
      accessor: 'title' as keyof AccessibilityIssue,
      sortable: true,
      render: (value: string, row: AccessibilityIssue) => (
        <div>
          <div
            style={{
              fontWeight: theme.typography.fontWeight.medium,
              color: theme.colors.text.primary,
              cursor: 'pointer',
            }}
            onClick={() => handleIssueClick(row)}
            role="button"
            tabIndex={0}
            onKeyPress={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                handleIssueClick(row);
              }
            }}
          >
            {value}
          </div>
          <div
            style={{
              fontSize: theme.typography.fontSize.xs,
              color: theme.colors.text.tertiary,
              marginTop: theme.spacing[1],
            }}
          >
            {row.category.replace(/-/g, ' ')}
          </div>
        </div>
      ),
    },
    {
      id: 'wcagCriteria',
      header: 'WCAG',
      accessor: (row: AccessibilityIssue) => row.wcagCriteria.join(', '),
      width: '150px',
      render: (value: string) => (
        <div style={{ fontSize: theme.typography.fontSize.sm }}>{value}</div>
      ),
    },
    {
      id: 'status',
      header: 'Status',
      accessor: 'status' as keyof AccessibilityIssue,
      width: '120px',
      sortable: true,
      render: (value: string) => {
        const statusColors: Record<string, string> = {
          open: theme.colors.status.warning,
          'in-progress': theme.colors.interactive.primary,
          fixed: theme.colors.status.success,
          'wont-fix': theme.colors.text.disabled,
          'false-positive': theme.colors.text.tertiary,
        };

        return (
          <div
            style={{
              display: 'inline-block',
              padding: `${theme.spacing[1]} ${theme.spacing[2]}`,
              backgroundColor: statusColors[value] || theme.colors.background.secondary,
              color: '#ffffff',
              borderRadius: theme.borderRadius.base,
              fontSize: theme.typography.fontSize.xs,
              textTransform: 'capitalize',
            }}
          >
            {value.replace(/-/g, ' ')}
          </div>
        );
      },
    },
    {
      id: 'detectedAt',
      header: 'Detected',
      accessor: 'detectedAt' as keyof AccessibilityIssue,
      width: '150px',
      sortable: true,
      render: (value: Date) => (
        <div style={{ fontSize: theme.typography.fontSize.sm }}>
          {new Date(value).toLocaleDateString()}
        </div>
      ),
    },
    {
      id: 'actions',
      header: 'Actions',
      accessor: () => null,
      width: '200px',
      render: (_: any, row: AccessibilityIssue) => (
        <div style={{ display: 'flex', gap: theme.spacing[1] }}>
          <Button
            size="sm"
            variant="ghost"
            onClick={(e) => {
              e.stopPropagation();
              highlight(row);
            }}
            title="Highlight element"
            aria-label="Highlight element in page"
          >
            🎯
          </Button>
          <Button
            size="sm"
            variant="success"
            onClick={(e) => {
              e.stopPropagation();
              handleMarkAsFixed(row.id);
            }}
            disabled={row.status === 'fixed'}
            title="Mark as fixed"
            aria-label="Mark issue as fixed"
          >
            ✓
          </Button>
          <Button
            size="sm"
            variant="danger"
            onClick={(e) => {
              e.stopPropagation();
              handleDelete(row.id);
            }}
            title="Delete issue"
            aria-label="Delete issue"
          >
            🗑
          </Button>
        </div>
      ),
    },
  ];

  return (
    <div style={{ padding: theme.spacing[6] }}>
      {/* Header */}
      <div style={{ marginBottom: theme.spacing[6] }}>
        <h1
          style={{
            fontSize: theme.typography.fontSize['2xl'],
            fontWeight: theme.typography.fontWeight.bold,
            color: theme.colors.text.primary,
            marginBottom: theme.spacing[2],
          }}
        >
          Accessibility Issues
        </h1>
        <p style={{ color: theme.colors.text.secondary }}>
          {filteredIssues.length} issue{filteredIssues.length !== 1 ? 's' : ''} found
        </p>
      </div>

      {/* Filters */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
          gap: theme.spacing[3],
          marginBottom: theme.spacing[4],
        }}
      >
        <Input
          type="text"
          placeholder="Search issues..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          aria-label="Search issues"
        />

        <Select
          value={selectedLevels}
          onChange={(e) => {
            const options = Array.from(e.target.selectedOptions, option => option.value as IssueLevel);
            setSelectedLevels(options);
          }}
          multiple
          aria-label="Filter by severity level"
        >
          <option value="">All Severities</option>
          <option value={IssueLevel.Critical}>Critical</option>
          <option value={IssueLevel.Serious}>Serious</option>
          <option value={IssueLevel.Moderate}>Moderate</option>
          <option value={IssueLevel.Minor}>Minor</option>
        </Select>

        <Select
          value={selectedStatus}
          onChange={(e) => {
            const options = Array.from(e.target.selectedOptions, option => option.value);
            setSelectedStatus(options);
          }}
          multiple
          aria-label="Filter by status"
        >
          <option value="">All Statuses</option>
          <option value="open">Open</option>
          <option value="in-progress">In Progress</option>
          <option value="fixed">Fixed</option>
          <option value="wont-fix">Won't Fix</option>
          <option value="false-positive">False Positive</option>
        </Select>

        <Button
          variant="ghost"
          onClick={() => {
            setSelectedLevels([]);
            setSelectedCategories([]);
            setSelectedStatus([]);
            setSearchTerm('');
          }}
          aria-label="Clear all filters"
        >
          Clear Filters
        </Button>
      </div>

      {/* Bulk Actions */}
      {showBulkActions && selectedIssues.length > 0 && (
        <div
          style={{
            padding: theme.spacing[4],
            backgroundColor: theme.colors.background.secondary,
            borderRadius: theme.borderRadius.md,
            marginBottom: theme.spacing[4],
            display: 'flex',
            gap: theme.spacing[3],
            alignItems: 'center',
          }}
          role="toolbar"
          aria-label="Bulk actions toolbar"
        >
          <span style={{ color: theme.colors.text.primary }}>
            {selectedIssues.length} selected
          </span>
          <Button
            size="sm"
            variant="primary"
            onClick={() => bulkUpdate({ status: 'in-progress' })}
          >
            Mark In Progress
          </Button>
          <Button
            size="sm"
            variant="success"
            onClick={() => bulkUpdate({ status: 'fixed', fixedAt: new Date() })}
          >
            Mark Fixed
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={deselectAll}
          >
            Deselect All
          </Button>
        </div>
      )}

      {/* Issues Table */}
      <Table
        columns={columns}
        data={filteredIssues}
        rowKey="id"
        selectable={showBulkActions}
        selectedRows={selectedIssues}
        onSelectionChange={(keys) => {
          if (keys.length === 0) deselectAll();
          else if (keys.length === filteredIssues.length) selectAll(keys);
        }}
        sortable
        sortColumn={sortConfig.field as string}
        sortDirection={sortConfig.direction}
        onSortChange={(columnId, direction) => {
          setSortConfig({
            field: columnId as keyof AccessibilityIssue,
            direction: direction || 'asc',
          });
        }}
        hoverable
        striped
        emptyState={
          <div style={{ textAlign: 'center', padding: theme.spacing[8] }}>
            <div style={{ fontSize: '48px', marginBottom: theme.spacing[4] }}>✅</div>
            <h3 style={{ marginBottom: theme.spacing[2] }}>No Issues Found</h3>
            <p style={{ color: theme.colors.text.secondary }}>
              Great job! Your application is accessible.
            </p>
          </div>
        }
      />

      {/* Issue Details Modal */}
      {selectedIssue && (
        <Modal
          isOpen={showDetailsModal}
          onClose={handleCloseDetails}
          title={selectedIssue.title}
          size="large"
        >
          <IssueDetails
            issue={selectedIssue}
            onMarkAsFixed={() => handleMarkAsFixed(selectedIssue.id)}
            onMarkAsFalsePositive={(reason) => handleMarkAsFalsePositive(selectedIssue.id, reason)}
            onClose={handleCloseDetails}
          />
        </Modal>
      )}
    </div>
  );
}

// Issue Details Component
function IssueDetails({
  issue,
  onMarkAsFixed,
  onMarkAsFalsePositive,
  onClose,
}: {
  issue: AccessibilityIssue;
  onMarkAsFixed: () => void;
  onMarkAsFalsePositive: (reason: string) => void;
  onClose: () => void;
}) {
  const { theme } = useTheme();
  const [falsePositiveReason, setFalsePositiveReason] = useState('');
  const [showFalsePositiveInput, setShowFalsePositiveInput] = useState(false);

  const levelColors = {
    [IssueLevel.Critical]: '#dc2626',
    [IssueLevel.Serious]: '#ea580c',
    [IssueLevel.Moderate]: '#f59e0b',
    [IssueLevel.Minor]: '#10b981',
  };

  return (
    <div style={{ padding: theme.spacing[4] }}>
      {/* Severity Badge */}
      <div
        style={{
          display: 'inline-block',
          padding: `${theme.spacing[2]} ${theme.spacing[4]}`,
          backgroundColor: levelColors[issue.level],
          color: '#ffffff',
          borderRadius: theme.borderRadius.base,
          fontSize: theme.typography.fontSize.sm,
          fontWeight: theme.typography.fontWeight.semibold,
          textTransform: 'uppercase',
          marginBottom: theme.spacing[4],
        }}
      >
        {issue.level} Severity
      </div>

      {/* Description */}
      <div style={{ marginBottom: theme.spacing[4] }}>
        <h3
          style={{
            fontSize: theme.typography.fontSize.lg,
            fontWeight: theme.typography.fontWeight.semibold,
            marginBottom: theme.spacing[2],
          }}
        >
          Description
        </h3>
        <p style={{ color: theme.colors.text.secondary }}>{issue.description}</p>
      </div>

      {/* WCAG Criteria */}
      <div style={{ marginBottom: theme.spacing[4] }}>
        <h3
          style={{
            fontSize: theme.typography.fontSize.lg,
            fontWeight: theme.typography.fontWeight.semibold,
            marginBottom: theme.spacing[2],
          }}
        >
          WCAG Criteria
        </h3>
        <div style={{ display: 'flex', gap: theme.spacing[2], flexWrap: 'wrap' }}>
          {issue.wcagCriteria.map((criteria) => (
            <div
              key={criteria}
              style={{
                padding: `${theme.spacing[1]} ${theme.spacing[3]}`,
                backgroundColor: theme.colors.background.secondary,
                borderRadius: theme.borderRadius.full,
                fontSize: theme.typography.fontSize.sm,
              }}
            >
              {criteria}
            </div>
          ))}
        </div>
      </div>

      {/* Code Snippet */}
      {issue.codeSnippet && (
        <div style={{ marginBottom: theme.spacing[4] }}>
          <h3
            style={{
              fontSize: theme.typography.fontSize.lg,
              fontWeight: theme.typography.fontWeight.semibold,
              marginBottom: theme.spacing[2],
            }}
          >
            Current Code
          </h3>
          <pre
            style={{
              padding: theme.spacing[3],
              backgroundColor: theme.colors.background.tertiary,
              borderRadius: theme.borderRadius.md,
              overflow: 'auto',
              fontSize: theme.typography.fontSize.sm,
              fontFamily: theme.typography.fontFamily.mono,
            }}
          >
            <code>{issue.codeSnippet}</code>
          </pre>
        </div>
      )}

      {/* Suggested Fix */}
      {issue.suggestedFix && (
        <div style={{ marginBottom: theme.spacing[4] }}>
          <h3
            style={{
              fontSize: theme.typography.fontSize.lg,
              fontWeight: theme.typography.fontWeight.semibold,
              marginBottom: theme.spacing[2],
            }}
          >
            Suggested Fix
          </h3>
          <p
            style={{
              padding: theme.spacing[3],
              backgroundColor: theme.colors.status.success + '20',
              borderLeft: `4px solid ${theme.colors.status.success}`,
              borderRadius: theme.borderRadius.md,
              color: theme.colors.text.primary,
            }}
          >
            {issue.suggestedFix}
          </p>
        </div>
      )}

      {/* Fixed Code Snippet */}
      {issue.fixedCodeSnippet && (
        <div style={{ marginBottom: theme.spacing[4] }}>
          <h3
            style={{
              fontSize: theme.typography.fontSize.lg,
              fontWeight: theme.typography.fontWeight.semibold,
              marginBottom: theme.spacing[2],
            }}
          >
            Fixed Code
          </h3>
          <pre
            style={{
              padding: theme.spacing[3],
              backgroundColor: theme.colors.status.success + '10',
              border: `1px solid ${theme.colors.status.success}`,
              borderRadius: theme.borderRadius.md,
              overflow: 'auto',
              fontSize: theme.typography.fontSize.sm,
              fontFamily: theme.typography.fontFamily.mono,
            }}
          >
            <code>{issue.fixedCodeSnippet}</code>
          </pre>
        </div>
      )}

      {/* Actions */}
      <div
        style={{
          display: 'flex',
          gap: theme.spacing[3],
          marginTop: theme.spacing[6],
          paddingTop: theme.spacing[4],
          borderTop: `1px solid ${theme.colors.border.primary}`,
        }}
      >
        <Button variant="success" onClick={onMarkAsFixed} disabled={issue.status === 'fixed'}>
          Mark as Fixed
        </Button>
        <Button
          variant="secondary"
          onClick={() => setShowFalsePositiveInput(!showFalsePositiveInput)}
        >
          Mark as False Positive
        </Button>
        <Button variant="ghost" onClick={onClose}>
          Close
        </Button>
      </div>

      {/* False Positive Input */}
      {showFalsePositiveInput && (
        <div style={{ marginTop: theme.spacing[4] }}>
          <Input
            type="text"
            placeholder="Enter reason for marking as false positive..."
            value={falsePositiveReason}
            onChange={(e) => setFalsePositiveReason(e.target.value)}
            style={{ marginBottom: theme.spacing[2] }}
          />
          <Button
            variant="primary"
            onClick={() => {
              if (falsePositiveReason.trim()) {
                onMarkAsFalsePositive(falsePositiveReason);
              }
            }}
            disabled={!falsePositiveReason.trim()}
          >
            Submit
          </Button>
        </div>
      )}
    </div>
  );
}
