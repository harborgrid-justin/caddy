/**
 * Accessibility Module Exports
 *
 * Central export file for all accessibility components, hooks, and types.
 */

// Components
export { AccessibilityDashboard } from './AccessibilityDashboard';
export { IssueExplorer } from './IssueExplorer';
export { ComplianceReport } from './ComplianceReport';
export {
  AccessibilityProvider,
  withAccessibility,
  AccessibilityContext,
} from './AccessibilityProvider';

// Hooks
export {
  useAccessibility,
  useFilteredIssues,
  useIssueStats,
  useCompliance,
  useScanStatus,
  useRealtimeScan,
  useTrendData,
  useBulkIssueOperations,
  useIssueHighlight,
  useAccessibilityScore,
  useAccessibilitySettings,
  useDebouncedSearch,
} from './useAccessibility';

// Types
export type {
  AccessibilityIssue,
  AccessibilityScore,
  ComplianceStatus,
  ScanResult,
  ScanConfig,
  AccessibilitySettings,
  IssueFilter,
  IssueSortConfig,
  ReportConfig,
  ExportOptions,
  AccessibilityContextValue,
  TrendDataPoint,
  AccessibilityTrend,
  QuickAction,
} from './types';

export {
  IssueLevel,
  IssueCategory,
  ComplianceStandard,
  ReportFormat,
  ScanStatus,
} from './types';

// Default export for convenience
export default {
  AccessibilityDashboard,
  IssueExplorer,
  ComplianceReport,
  AccessibilityProvider,
  withAccessibility,
  useAccessibility,
};
