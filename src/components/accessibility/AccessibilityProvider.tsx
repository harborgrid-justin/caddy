/**
 * Accessibility Provider
 *
 * React context provider for accessibility functionality with multi-tenant support.
 */

import React, { createContext, useState, useEffect, useCallback, ReactNode } from 'react';
import type {
  AccessibilityContextValue,
  AccessibilityIssue,
  AccessibilityScore,
  ComplianceStatus,
  AccessibilityTrend,
  ScanResult,
  AccessibilitySettings,
  IssueFilter,
  IssueSortConfig,
  ScanConfig,
  ComplianceStandard,
  ReportConfig,
  ExportOptions,
} from './types';
import { IssueLevel, ScanStatus } from './types';

export const AccessibilityContext = createContext<AccessibilityContextValue | null>(null);

interface AccessibilityProviderProps {
  children: ReactNode;
  tenantId?: string;
  apiBaseUrl?: string;
}

export function AccessibilityProvider({
  children,
  tenantId,
  apiBaseUrl = '/api/accessibility'
}: AccessibilityProviderProps) {
  // State management
  const [issues, setIssues] = useState<AccessibilityIssue[]>([]);
  const [score, setScore] = useState<AccessibilityScore | null>(null);
  const [complianceStatus, setComplianceStatus] = useState<ComplianceStatus[]>([]);
  const [trends, setTrends] = useState<AccessibilityTrend[]>([]);
  const [currentScan, setCurrentScan] = useState<ScanResult | null>(null);
  const [scanHistory, setScanHistory] = useState<ScanResult[]>([]);
  const [settings, setSettings] = useState<AccessibilitySettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  // Build API URL with tenant support
  const buildUrl = useCallback((path: string) => {
    const base = apiBaseUrl;
    const tenant = tenantId ? `?tenantId=${tenantId}` : '';
    return `${base}${path}${tenant}`;
  }, [apiBaseUrl, tenantId]);

  // Initialize on mount
  useEffect(() => {
    const initialize = async () => {
      try {
        setLoading(true);
        await Promise.all([
          loadSettings(),
          loadScore(),
          loadIssues(),
          loadComplianceStatus(),
          loadScanHistory(),
        ]);
        setError(null);
      } catch (err) {
        setError(err as Error);
        console.error('Failed to initialize accessibility provider:', err);
      } finally {
        setLoading(false);
      }
    };

    initialize();
  }, [tenantId]);

  // Auto-refresh data
  useEffect(() => {
    if (!settings?.scanConfig.enabled) return;

    const interval = setInterval(() => {
      refreshData();
    }, 30000); // Refresh every 30 seconds

    return () => clearInterval(interval);
  }, [settings]);

  // Load functions
  const loadSettings = async () => {
    const response = await fetch(buildUrl('/settings'));
    if (response.ok) {
      const data = await response.json();
      setSettings({ ...data, tenantId });
    }
  };

  const loadScore = async () => {
    const response = await fetch(buildUrl('/score'));
    if (response.ok) {
      const data = await response.json();
      setScore(data);
    }
  };

  const loadIssues = async () => {
    const response = await fetch(buildUrl('/issues'));
    if (response.ok) {
      const data = await response.json();
      setIssues(data);
    }
  };

  const loadComplianceStatus = async () => {
    const response = await fetch(buildUrl('/compliance'));
    if (response.ok) {
      const data = await response.json();
      setComplianceStatus(data);
    }
  };

  const loadScanHistory = async () => {
    const response = await fetch(buildUrl('/scans'));
    if (response.ok) {
      const data = await response.json();
      setScanHistory(data);
    }
  };

  // Scanning methods
  const startScan = useCallback(async (config?: Partial<ScanConfig>) => {
    try {
      const response = await fetch(buildUrl('/scan/start'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config || {}),
      });

      if (!response.ok) {
        throw new Error('Failed to start scan');
      }

      const scan = await response.json();
      setCurrentScan(scan);

      // Poll for scan completion
      const pollInterval = setInterval(async () => {
        const statusResponse = await fetch(buildUrl(`/scan/${scan.id}`));
        if (statusResponse.ok) {
          const updatedScan = await statusResponse.json();
          setCurrentScan(updatedScan);

          if (updatedScan.status === ScanStatus.Completed || updatedScan.status === ScanStatus.Failed) {
            clearInterval(pollInterval);
            await refreshData();
          }
        }
      }, 2000);
    } catch (err) {
      setError(err as Error);
      throw err;
    }
  }, [buildUrl]);

  const stopScan = useCallback(async () => {
    if (!currentScan) return;

    try {
      const response = await fetch(buildUrl(`/scan/${currentScan.id}/stop`), {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error('Failed to stop scan');
      }

      setCurrentScan(null);
    } catch (err) {
      setError(err as Error);
      throw err;
    }
  }, [currentScan, buildUrl]);

  const getScanResult = useCallback(async (scanId: string): Promise<ScanResult> => {
    const response = await fetch(buildUrl(`/scan/${scanId}`));
    if (!response.ok) {
      throw new Error('Failed to fetch scan result');
    }
    return await response.json();
  }, [buildUrl]);

  // Issue management methods
  const getIssues = useCallback(async (
    filter?: IssueFilter,
    sort?: IssueSortConfig
  ): Promise<AccessibilityIssue[]> => {
    const params = new URLSearchParams();
    if (filter) {
      Object.entries(filter).forEach(([key, value]) => {
        if (value !== undefined) {
          params.append(key, JSON.stringify(value));
        }
      });
    }
    if (sort) {
      params.append('sort', JSON.stringify(sort));
    }

    const url = buildUrl(`/issues?${params.toString()}`);
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error('Failed to fetch issues');
    }

    const data = await response.json();
    setIssues(data);
    return data;
  }, [buildUrl]);

  const getIssueById = useCallback(async (issueId: string): Promise<AccessibilityIssue> => {
    const response = await fetch(buildUrl(`/issues/${issueId}`));
    if (!response.ok) {
      throw new Error('Failed to fetch issue');
    }
    return await response.json();
  }, [buildUrl]);

  const updateIssue = useCallback(async (
    issueId: string,
    updates: Partial<AccessibilityIssue>
  ) => {
    const response = await fetch(buildUrl(`/issues/${issueId}`), {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(updates),
    });

    if (!response.ok) {
      throw new Error('Failed to update issue');
    }

    // Update local state
    setIssues(prev => prev.map(issue =>
      issue.id === issueId ? { ...issue, ...updates } : issue
    ));
  }, [buildUrl]);

  const deleteIssue = useCallback(async (issueId: string) => {
    const response = await fetch(buildUrl(`/issues/${issueId}`), {
      method: 'DELETE',
    });

    if (!response.ok) {
      throw new Error('Failed to delete issue');
    }

    setIssues(prev => prev.filter(issue => issue.id !== issueId));
  }, [buildUrl]);

  const bulkUpdateIssues = useCallback(async (
    issueIds: string[],
    updates: Partial<AccessibilityIssue>
  ) => {
    const response = await fetch(buildUrl('/issues/bulk'), {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ issueIds, updates }),
    });

    if (!response.ok) {
      throw new Error('Failed to bulk update issues');
    }

    // Update local state
    setIssues(prev => prev.map(issue =>
      issueIds.includes(issue.id) ? { ...issue, ...updates } : issue
    ));
  }, [buildUrl]);

  const markIssueAsFixed = useCallback(async (issueId: string) => {
    await updateIssue(issueId, {
      status: 'fixed',
      fixedAt: new Date()
    });
  }, [updateIssue]);

  const markIssueAsFalsePositive = useCallback(async (
    issueId: string,
    reason: string
  ) => {
    await updateIssue(issueId, {
      status: 'false-positive',
      notes: reason
    });
  }, [updateIssue]);

  // Compliance methods
  const checkCompliance = useCallback(async (
    standard: ComplianceStandard
  ): Promise<ComplianceStatus> => {
    const response = await fetch(buildUrl(`/compliance/${standard}`));
    if (!response.ok) {
      throw new Error('Failed to check compliance');
    }
    return await response.json();
  }, [buildUrl]);

  const getComplianceReport = useCallback(async (standards: ComplianceStandard[]) => {
    const response = await fetch(buildUrl('/compliance/report'), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ standards }),
    });

    if (!response.ok) {
      throw new Error('Failed to generate compliance report');
    }

    return await response.json();
  }, [buildUrl]);

  // Trends & Analytics
  const getTrends = useCallback(async (
    period: '24h' | '7d' | '30d' | '90d' | '1y'
  ): Promise<AccessibilityTrend> => {
    const response = await fetch(buildUrl(`/trends/${period}`));
    if (!response.ok) {
      throw new Error('Failed to fetch trends');
    }
    return await response.json();
  }, [buildUrl]);

  const getScoreHistory = useCallback(async (days: number) => {
    const response = await fetch(buildUrl(`/score/history?days=${days}`));
    if (!response.ok) {
      throw new Error('Failed to fetch score history');
    }
    return await response.json();
  }, [buildUrl]);

  // Settings
  const updateSettings = useCallback(async (
    updates: Partial<AccessibilitySettings>
  ) => {
    const response = await fetch(buildUrl('/settings'), {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(updates),
    });

    if (!response.ok) {
      throw new Error('Failed to update settings');
    }

    const updatedSettings = await response.json();
    setSettings(updatedSettings);
  }, [buildUrl]);

  const resetSettings = useCallback(async () => {
    const response = await fetch(buildUrl('/settings/reset'), {
      method: 'POST',
    });

    if (!response.ok) {
      throw new Error('Failed to reset settings');
    }

    await loadSettings();
  }, [buildUrl]);

  // Reporting
  const generateReport = useCallback(async (config: ReportConfig): Promise<Blob> => {
    const response = await fetch(buildUrl('/report/generate'), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });

    if (!response.ok) {
      throw new Error('Failed to generate report');
    }

    return await response.blob();
  }, [buildUrl]);

  const scheduleReport = useCallback(async (config: ReportConfig) => {
    const response = await fetch(buildUrl('/report/schedule'), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });

    if (!response.ok) {
      throw new Error('Failed to schedule report');
    }
  }, [buildUrl]);

  const exportData = useCallback(async (options: ExportOptions): Promise<Blob> => {
    const response = await fetch(buildUrl('/export'), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(options),
    });

    if (!response.ok) {
      throw new Error('Failed to export data');
    }

    return await response.blob();
  }, [buildUrl]);

  // Element Highlighting
  const highlightElement = useCallback((selector: string) => {
    // Remove existing highlights
    document.querySelectorAll('.accessibility-highlight').forEach(el => {
      el.classList.remove('accessibility-highlight');
    });

    // Add highlight to target element
    const element = document.querySelector(selector);
    if (element) {
      element.classList.add('accessibility-highlight');
      element.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }, []);

  const unhighlightAll = useCallback(() => {
    document.querySelectorAll('.accessibility-highlight').forEach(el => {
      el.classList.remove('accessibility-highlight');
    });
  }, []);

  // Refresh all data
  const refreshData = useCallback(async () => {
    try {
      await Promise.all([
        loadScore(),
        loadIssues(),
        loadComplianceStatus(),
        loadScanHistory(),
      ]);
      setError(null);
    } catch (err) {
      setError(err as Error);
      console.error('Failed to refresh data:', err);
    }
  }, [buildUrl]);

  // Context value
  const value: AccessibilityContextValue = {
    // State
    issues,
    score,
    complianceStatus,
    trends,
    currentScan,
    scanHistory,
    settings,
    loading,
    error,

    // Scanning
    startScan,
    stopScan,
    getScanResult,

    // Issue Management
    getIssues,
    getIssueById,
    updateIssue,
    deleteIssue,
    bulkUpdateIssues,
    markIssueAsFixed,
    markIssueAsFalsePositive,

    // Compliance
    checkCompliance,
    getComplianceReport,

    // Trends & Analytics
    getTrends,
    getScoreHistory,

    // Settings
    updateSettings,
    resetSettings,

    // Reporting
    generateReport,
    scheduleReport,
    exportData,

    // Element Highlighting
    highlightElement,
    unhighlightAll,

    // Refresh
    refreshData,
  };

  return (
    <AccessibilityContext.Provider value={value}>
      {children}
    </AccessibilityContext.Provider>
  );
}

/**
 * Higher-order component to wrap components with accessibility provider
 */
export function withAccessibility<P extends object>(
  Component: React.ComponentType<P>
): React.ComponentType<P & { tenantId?: string }> {
  return function WrappedComponent(props: P & { tenantId?: string }) {
    return (
      <AccessibilityProvider tenantId={props.tenantId}>
        <Component {...props} />
      </AccessibilityProvider>
    );
  };
}
