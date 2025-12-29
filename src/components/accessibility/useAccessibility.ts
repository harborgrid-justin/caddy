/**
 * Accessibility Hooks
 *
 * Custom React hooks for accessibility features.
 */

import { useContext, useState, useEffect, useMemo, useCallback } from 'react';
import { AccessibilityContext } from './AccessibilityProvider';
import type {
  AccessibilityContextValue,
  AccessibilityIssue,
  IssueFilter,
  IssueSortConfig,
  IssueLevel,
  IssueCategory,
  ComplianceStandard,
} from './types';

/**
 * Main hook to access accessibility context
 */
export function useAccessibility(): AccessibilityContextValue {
  const context = useContext(AccessibilityContext);
  if (!context) {
    throw new Error('useAccessibility must be used within AccessibilityProvider');
  }
  return context;
}

/**
 * Hook for filtered and sorted issues
 */
export function useFilteredIssues(
  filter?: IssueFilter,
  sort?: IssueSortConfig
) {
  const { issues } = useAccessibility();
  const [filteredIssues, setFilteredIssues] = useState<AccessibilityIssue[]>([]);

  useEffect(() => {
    let result = [...issues];

    // Apply filters
    if (filter) {
      if (filter.levels && filter.levels.length > 0) {
        result = result.filter(issue => filter.levels!.includes(issue.level));
      }

      if (filter.categories && filter.categories.length > 0) {
        result = result.filter(issue => filter.categories!.includes(issue.category));
      }

      if (filter.status && filter.status.length > 0) {
        result = result.filter(issue => filter.status!.includes(issue.status));
      }

      if (filter.searchQuery) {
        const query = filter.searchQuery.toLowerCase();
        result = result.filter(issue =>
          issue.title.toLowerCase().includes(query) ||
          issue.description.toLowerCase().includes(query) ||
          issue.category.toLowerCase().includes(query)
        );
      }

      if (filter.componentName) {
        result = result.filter(issue => issue.componentName === filter.componentName);
      }

      if (filter.pageUrl) {
        result = result.filter(issue => issue.pageUrl === filter.pageUrl);
      }

      if (filter.assignee) {
        result = result.filter(issue => issue.assignee === filter.assignee);
      }

      if (filter.dateRange) {
        result = result.filter(issue => {
          const issueDate = new Date(issue.detectedAt);
          return issueDate >= filter.dateRange!.start && issueDate <= filter.dateRange!.end;
        });
      }
    }

    // Apply sorting
    if (sort) {
      result.sort((a, b) => {
        const aValue = a[sort.field];
        const bValue = b[sort.field];

        if (aValue === bValue) return 0;

        const comparison = aValue < bValue ? -1 : 1;
        return sort.direction === 'asc' ? comparison : -comparison;
      });
    }

    setFilteredIssues(result);
  }, [issues, filter, sort]);

  return filteredIssues;
}

/**
 * Hook for issue statistics
 */
export function useIssueStats() {
  const { issues, score } = useAccessibility();

  return useMemo(() => {
    const totalIssues = issues.length;
    const openIssues = issues.filter(i => i.status === 'open').length;
    const fixedIssues = issues.filter(i => i.status === 'fixed').length;
    const inProgressIssues = issues.filter(i => i.status === 'in-progress').length;

    const byLevel = issues.reduce((acc, issue) => {
      acc[issue.level] = (acc[issue.level] || 0) + 1;
      return acc;
    }, {} as Record<IssueLevel, number>);

    const byCategory = issues.reduce((acc, issue) => {
      acc[issue.category] = (acc[issue.category] || 0) + 1;
      return acc;
    }, {} as Record<IssueCategory, number>);

    return {
      totalIssues,
      openIssues,
      fixedIssues,
      inProgressIssues,
      byLevel,
      byCategory,
      score: score?.overall || 0,
      trend: score?.trend || 'stable',
    };
  }, [issues, score]);
}

/**
 * Hook for compliance information
 */
export function useCompliance(standard?: ComplianceStandard) {
  const { complianceStatus } = useAccessibility();

  return useMemo(() => {
    if (standard) {
      return complianceStatus.find(c => c.standard === standard);
    }
    return complianceStatus;
  }, [complianceStatus, standard]);
}

/**
 * Hook for scanning status
 */
export function useScanStatus() {
  const { currentScan, scanHistory } = useAccessibility();

  return useMemo(() => {
    const isScanning = currentScan?.status === 'running';
    const lastScan = scanHistory[0];
    const lastScanDate = lastScan?.timestamp;
    const lastScanDuration = lastScan?.duration;

    return {
      isScanning,
      currentScan,
      lastScan,
      lastScanDate,
      lastScanDuration,
      scanHistory,
    };
  }, [currentScan, scanHistory]);
}

/**
 * Hook for real-time scanning
 */
export function useRealtimeScan(autoStart = false) {
  const { startScan, stopScan, currentScan } = useAccessibility();
  const [scanning, setScanning] = useState(false);

  useEffect(() => {
    if (autoStart && !currentScan) {
      handleStart();
    }
  }, [autoStart]);

  const handleStart = useCallback(async () => {
    setScanning(true);
    try {
      await startScan();
    } catch (err) {
      console.error('Scan failed:', err);
      setScanning(false);
    }
  }, [startScan]);

  const handleStop = useCallback(async () => {
    await stopScan();
    setScanning(false);
  }, [stopScan]);

  useEffect(() => {
    if (currentScan?.status === 'completed' || currentScan?.status === 'failed') {
      setScanning(false);
    }
  }, [currentScan]);

  return {
    scanning,
    startScan: handleStart,
    stopScan: handleStop,
    currentScan,
  };
}

/**
 * Hook for trending data
 */
export function useTrendData(period: '24h' | '7d' | '30d' | '90d' | '1y' = '7d') {
  const { getTrends } = useAccessibility();
  const [trendData, setTrendData] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchTrends = async () => {
      setLoading(true);
      try {
        const data = await getTrends(period);
        setTrendData(data);
      } catch (err) {
        console.error('Failed to fetch trends:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchTrends();
  }, [period, getTrends]);

  return { trendData, loading };
}

/**
 * Hook for bulk issue operations
 */
export function useBulkIssueOperations() {
  const { bulkUpdateIssues } = useAccessibility();
  const [selectedIssues, setSelectedIssues] = useState<string[]>([]);

  const selectIssue = useCallback((issueId: string) => {
    setSelectedIssues(prev => [...prev, issueId]);
  }, []);

  const deselectIssue = useCallback((issueId: string) => {
    setSelectedIssues(prev => prev.filter(id => id !== issueId));
  }, []);

  const toggleIssue = useCallback((issueId: string) => {
    setSelectedIssues(prev =>
      prev.includes(issueId)
        ? prev.filter(id => id !== issueId)
        : [...prev, issueId]
    );
  }, []);

  const selectAll = useCallback((issueIds: string[]) => {
    setSelectedIssues(issueIds);
  }, []);

  const deselectAll = useCallback(() => {
    setSelectedIssues([]);
  }, []);

  const bulkUpdate = useCallback(async (updates: Partial<AccessibilityIssue>) => {
    if (selectedIssues.length === 0) return;
    await bulkUpdateIssues(selectedIssues, updates);
    setSelectedIssues([]);
  }, [selectedIssues, bulkUpdateIssues]);

  return {
    selectedIssues,
    selectIssue,
    deselectIssue,
    toggleIssue,
    selectAll,
    deselectAll,
    bulkUpdate,
  };
}

/**
 * Hook for issue highlighting
 */
export function useIssueHighlight() {
  const { highlightElement, unhighlightAll } = useAccessibility();
  const [highlightedIssueId, setHighlightedIssueId] = useState<string | null>(null);

  const highlight = useCallback((issue: AccessibilityIssue) => {
    if (issue.element?.selector) {
      highlightElement(issue.element.selector);
      setHighlightedIssueId(issue.id);
    }
  }, [highlightElement]);

  const unhighlight = useCallback(() => {
    unhighlightAll();
    setHighlightedIssueId(null);
  }, [unhighlightAll]);

  return {
    highlightedIssueId,
    highlight,
    unhighlight,
  };
}

/**
 * Hook for accessibility score with color coding
 */
export function useAccessibilityScore() {
  const { score } = useAccessibility();

  return useMemo(() => {
    if (!score) return null;

    const getScoreColor = (scoreValue: number) => {
      if (scoreValue >= 90) return 'success';
      if (scoreValue >= 70) return 'warning';
      return 'error';
    };

    const getScoreGrade = (scoreValue: number) => {
      if (scoreValue >= 90) return 'A';
      if (scoreValue >= 80) return 'B';
      if (scoreValue >= 70) return 'C';
      if (scoreValue >= 60) return 'D';
      return 'F';
    };

    return {
      ...score,
      color: getScoreColor(score.overall),
      grade: getScoreGrade(score.overall),
    };
  }, [score]);
}

/**
 * Hook for accessibility settings
 */
export function useAccessibilitySettings() {
  const { settings, updateSettings, resetSettings } = useAccessibility();

  const updateSetting = useCallback(async (path: string, value: any) => {
    if (!settings) return;

    const keys = path.split('.');
    const updates = { ...settings };
    let current: any = updates;

    for (let i = 0; i < keys.length - 1; i++) {
      current = current[keys[i]];
    }

    current[keys[keys.length - 1]] = value;
    await updateSettings(updates);
  }, [settings, updateSettings]);

  return {
    settings,
    updateSettings,
    updateSetting,
    resetSettings,
  };
}

/**
 * Hook for debounced search
 */
export function useDebouncedSearch(delay = 300) {
  const [searchTerm, setSearchTerm] = useState('');
  const [debouncedTerm, setDebouncedTerm] = useState('');

  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedTerm(searchTerm);
    }, delay);

    return () => clearTimeout(timer);
  }, [searchTerm, delay]);

  return {
    searchTerm,
    setSearchTerm,
    debouncedTerm,
  };
}
