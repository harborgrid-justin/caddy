/**
 * Suggestion Panel - Display and manage AI-powered accessibility suggestions
 *
 * Features:
 * - Auto-fix previews with code diff
 * - Batch application of fixes
 * - Filtering and sorting suggestions
 * - Confidence scoring visualization
 * - One-click fix application
 */

import React, { useState, useMemo, useCallback } from 'react';
import type { SuggestionPanelProps, AutoFix, SuggestionConfidence } from './types';

type FilterOption = 'all' | 'high' | 'medium' | 'low';
type SortOption = 'confidence' | 'type' | 'recent';

export function SuggestionPanel({
  suggestions,
  onApply,
  onDismiss,
  onPreview,
  loading = false,
}: SuggestionPanelProps) {
  const [filter, setFilter] = useState<FilterOption>('all');
  const [sortBy, setSortBy] = useState<SortOption>('confidence');
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [expandedId, setExpandedId] = useState<string | null>(null);

  // Filter and sort suggestions
  const filteredAndSorted = useMemo(() => {
    let result = [...suggestions];

    // Apply filter
    if (filter !== 'all') {
      const targetConfidence = filter.charAt(0).toUpperCase() + filter.slice(1) as SuggestionConfidence;
      result = result.filter((s) => s.confidence === targetConfidence);
    }

    // Apply sort
    result.sort((a, b) => {
      switch (sortBy) {
        case 'confidence': {
          const confidenceOrder = { High: 3, Medium: 2, Low: 1 };
          return confidenceOrder[b.confidence] - confidenceOrder[a.confidence];
        }
        case 'type':
          return a.fixType.localeCompare(b.fixType);
        case 'recent':
          return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
        default:
          return 0;
      }
    });

    return result;
  }, [suggestions, filter, sortBy]);

  // Statistics
  const stats = useMemo(() => {
    const total = suggestions.length;
    const high = suggestions.filter((s) => s.confidence === 'High').length;
    const medium = suggestions.filter((s) => s.confidence === 'Medium').length;
    const low = suggestions.filter((s) => s.confidence === 'Low').length;
    const requiresReview = suggestions.filter((s) => s.requiresManualReview).length;

    return { total, high, medium, low, requiresReview };
  }, [suggestions]);

  // Handle selection
  const handleToggleSelection = useCallback((id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const handleSelectAll = useCallback(() => {
    if (selectedIds.size === filteredAndSorted.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(filteredAndSorted.map((s) => s.issueId)));
    }
  }, [filteredAndSorted, selectedIds.size]);

  // Handle batch actions
  const handleApplySelected = useCallback(() => {
    const selectedSuggestions = suggestions.filter((s) => selectedIds.has(s.issueId));
    selectedSuggestions.forEach((s) => onApply(s));
    setSelectedIds(new Set());
  }, [suggestions, selectedIds, onApply]);

  const handleDismissSelected = useCallback(() => {
    selectedIds.forEach((id) => onDismiss(id));
    setSelectedIds(new Set());
  }, [selectedIds, onDismiss]);

  // Handle expand/collapse
  const handleToggleExpand = useCallback((id: string) => {
    setExpandedId((prev) => (prev === id ? null : id));
  }, []);

  if (loading) {
    return (
      <div className="suggestion-panel loading">
        <div className="loading-spinner" />
        <p>Analyzing code for accessibility improvements...</p>
      </div>
    );
  }

  if (suggestions.length === 0) {
    return (
      <div className="suggestion-panel empty">
        <div className="empty-state">
          <div className="empty-icon">✨</div>
          <h3>No suggestions available</h3>
          <p>Run an accessibility scan to get AI-powered suggestions.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="suggestion-panel">
      {/* Header */}
      <div className="panel-header">
        <div className="header-title">
          <h2>AI Suggestions</h2>
          <span className="suggestion-count">{stats.total} suggestions</span>
        </div>

        {/* Statistics */}
        <div className="stats-bar">
          <StatBadge label="High" count={stats.high} color="green" />
          <StatBadge label="Medium" count={stats.medium} color="orange" />
          <StatBadge label="Low" count={stats.low} color="red" />
          {stats.requiresReview > 0 && (
            <StatBadge label="Review needed" count={stats.requiresReview} color="purple" />
          )}
        </div>
      </div>

      {/* Controls */}
      <div className="panel-controls">
        <div className="control-group">
          <label htmlFor="filter-select">Filter:</label>
          <select
            id="filter-select"
            value={filter}
            onChange={(e) => setFilter(e.target.value as FilterOption)}
            className="filter-select"
          >
            <option value="all">All suggestions</option>
            <option value="high">High confidence</option>
            <option value="medium">Medium confidence</option>
            <option value="low">Low confidence</option>
          </select>
        </div>

        <div className="control-group">
          <label htmlFor="sort-select">Sort by:</label>
          <select
            id="sort-select"
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as SortOption)}
            className="sort-select"
          >
            <option value="confidence">Confidence</option>
            <option value="type">Type</option>
            <option value="recent">Most recent</option>
          </select>
        </div>

        {selectedIds.size > 0 && (
          <div className="batch-actions">
            <span className="selected-count">{selectedIds.size} selected</span>
            <button onClick={handleApplySelected} className="apply-batch-button">
              Apply all
            </button>
            <button onClick={handleDismissSelected} className="dismiss-batch-button">
              Dismiss all
            </button>
          </div>
        )}
      </div>

      {/* Selection controls */}
      {filteredAndSorted.length > 0 && (
        <div className="selection-controls">
          <label className="select-all-checkbox">
            <input
              type="checkbox"
              checked={selectedIds.size === filteredAndSorted.length}
              onChange={handleSelectAll}
            />
            <span>Select all ({filteredAndSorted.length})</span>
          </label>
        </div>
      )}

      {/* Suggestions List */}
      <div className="suggestions-list">
        {filteredAndSorted.map((suggestion) => (
          <SuggestionItem
            key={suggestion.issueId}
            suggestion={suggestion}
            isSelected={selectedIds.has(suggestion.issueId)}
            isExpanded={expandedId === suggestion.issueId}
            onToggleSelection={() => handleToggleSelection(suggestion.issueId)}
            onToggleExpand={() => handleToggleExpand(suggestion.issueId)}
            onApply={() => onApply(suggestion)}
            onDismiss={() => onDismiss(suggestion.issueId)}
            onPreview={onPreview}
          />
        ))}
      </div>
    </div>
  );
}

// Stat Badge Component
interface StatBadgeProps {
  label: string;
  count: number;
  color: string;
}

function StatBadge({ label, count, color }: StatBadgeProps) {
  return (
    <div className={`stat-badge stat-${color}`}>
      <span className="stat-count">{count}</span>
      <span className="stat-label">{label}</span>
    </div>
  );
}

// Suggestion Item Component
interface SuggestionItemProps {
  suggestion: AutoFix;
  isSelected: boolean;
  isExpanded: boolean;
  onToggleSelection: () => void;
  onToggleExpand: () => void;
  onApply: () => void;
  onDismiss: () => void;
  onPreview?: (suggestion: AutoFix) => void;
}

function SuggestionItem({
  suggestion,
  isSelected,
  isExpanded,
  onToggleSelection,
  onToggleExpand,
  onApply,
  onDismiss,
  onPreview,
}: SuggestionItemProps) {
  const confidenceClass = suggestion.confidence.toLowerCase();

  return (
    <div
      className={`suggestion-item ${isSelected ? 'selected' : ''} ${
        isExpanded ? 'expanded' : ''
      }`}
    >
      {/* Header */}
      <div className="suggestion-item-header">
        <label className="selection-checkbox">
          <input
            type="checkbox"
            checked={isSelected}
            onChange={onToggleSelection}
            aria-label={`Select ${suggestion.fixType}`}
          />
        </label>

        <div className="suggestion-info">
          <h3 className="suggestion-title">{formatFixType(suggestion.fixType)}</h3>
          <p className="suggestion-explanation">{suggestion.explanation}</p>
        </div>

        <div className="suggestion-badges">
          <span className={`confidence-badge confidence-${confidenceClass}`}>
            {suggestion.confidence}
          </span>
          {suggestion.requiresManualReview && (
            <span className="review-badge" title="Manual review recommended">
              ⚠️
            </span>
          )}
        </div>
      </div>

      {/* Expandable Details */}
      {isExpanded && (
        <div className="suggestion-details">
          {/* Code Diff */}
          <div className="code-diff">
            <div className="diff-section">
              <label className="diff-label">Original:</label>
              <pre className="code-block original">
                <code>{suggestion.originalCode}</code>
              </pre>
            </div>

            <div className="diff-arrow">→</div>

            <div className="diff-section">
              <label className="diff-label">Fixed:</label>
              <pre className="code-block fixed">
                <code>{suggestion.fixedCode}</code>
              </pre>
            </div>
          </div>

          {/* WCAG Information */}
          {suggestion.wcagCriteria.length > 0 && (
            <div className="wcag-info">
              <label className="info-label">WCAG Criteria:</label>
              <div className="wcag-badges">
                {suggestion.wcagCriteria.map((criterion) => (
                  <span key={criterion} className="wcag-badge">
                    {criterion}
                  </span>
                ))}
              </div>
            </div>
          )}

          {/* Additional Info */}
          <div className="additional-info">
            <div className="info-item">
              <span className="info-label">Issue ID:</span>
              <span className="info-value">{suggestion.issueId}</span>
            </div>
            <div className="info-item">
              <span className="info-label">Generated:</span>
              <span className="info-value">
                {new Date(suggestion.timestamp).toLocaleString()}
              </span>
            </div>
          </div>
        </div>
      )}

      {/* Actions */}
      <div className="suggestion-actions">
        <button
          className="toggle-button"
          onClick={onToggleExpand}
          aria-expanded={isExpanded}
        >
          {isExpanded ? 'Hide details' : 'Show details'}
        </button>

        {onPreview && (
          <button
            className="preview-button"
            onClick={() => onPreview(suggestion)}
          >
            Preview
          </button>
        )}

        <button className="dismiss-button" onClick={onDismiss}>
          Dismiss
        </button>

        <button className="apply-button primary" onClick={onApply}>
          Apply fix
        </button>
      </div>
    </div>
  );
}

// Utility function to format fix type
function formatFixType(fixType: string): string {
  return fixType
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

// Export additional utilities
export { formatFixType };
