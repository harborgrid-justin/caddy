/**
 * CADDY v0.3.0 - Accessibility Scanner Types
 * Enterprise-grade browser extension types
 */

// ============================================================================
// Core Types
// ============================================================================

export type SeverityLevel = 'critical' | 'serious' | 'moderate' | 'minor';
export type IssueCategory = 'perceivable' | 'operable' | 'understandable' | 'robust';
export type WCAGLevel = 'A' | 'AA' | 'AAA';
export type ScanStatus = 'idle' | 'scanning' | 'complete' | 'error';

// ============================================================================
// Accessibility Issue
// ============================================================================

export interface AccessibilityIssue {
  id: string;
  severity: SeverityLevel;
  category: IssueCategory;
  wcagCriteria: string[];
  wcagLevel: WCAGLevel;
  title: string;
  description: string;
  element: ElementInfo;
  selector: string;
  snippet: string;
  impact: string;
  suggestion: string;
  helpUrl: string;
  timestamp: number;
}

export interface ElementInfo {
  tagName: string;
  id?: string;
  className?: string;
  role?: string;
  ariaLabel?: string;
  textContent?: string;
  attributes: Record<string, string>;
  boundingRect: DOMRect;
  xpath: string;
}

// ============================================================================
// Scan Results
// ============================================================================

export interface ScanResult {
  url: string;
  timestamp: number;
  duration: number;
  status: ScanStatus;
  issues: AccessibilityIssue[];
  summary: ScanSummary;
  metadata: ScanMetadata;
}

export interface ScanSummary {
  total: number;
  critical: number;
  serious: number;
  moderate: number;
  minor: number;
  passed: number;
  incomplete: number;
  byCategory: Record<IssueCategory, number>;
  wcagLevel: WCAGLevel;
  complianceScore: number;
}

export interface ScanMetadata {
  pageTitle: string;
  pageUrl: string;
  domNodes: number;
  scannerVersion: string;
  rules: string[];
  viewport: {
    width: number;
    height: number;
  };
}

// ============================================================================
// Configuration
// ============================================================================

export interface ScannerConfig {
  enabled: boolean;
  autoScan: boolean;
  wcagLevel: WCAGLevel;
  includeWarnings: boolean;
  highlightIssues: boolean;
  notifyOnIssues: boolean;
  rules: RuleConfig[];
  ignorePatterns: string[];
}

export interface RuleConfig {
  id: string;
  enabled: boolean;
  severity: SeverityLevel;
}

// ============================================================================
// User Settings
// ============================================================================

export interface UserSettings {
  apiKey?: string;
  apiEndpoint: string;
  syncEnabled: boolean;
  theme: 'light' | 'dark' | 'auto';
  notifications: NotificationSettings;
  scanning: ScanningSettings;
  keyboard: KeyboardSettings;
}

export interface NotificationSettings {
  enabled: boolean;
  onNewIssues: boolean;
  onScanComplete: boolean;
  severity: SeverityLevel[];
}

export interface ScanningSettings {
  autoScan: boolean;
  scanOnLoad: boolean;
  scanFrames: boolean;
  wcagLevel: WCAGLevel;
  maxIssues: number;
}

export interface KeyboardSettings {
  scanPage: string;
  toggleHighlights: string;
  nextIssue: string;
  previousIssue: string;
}

// ============================================================================
// Message Types
// ============================================================================

export type MessageType =
  | 'SCAN_PAGE'
  | 'SCAN_COMPLETE'
  | 'SCAN_ERROR'
  | 'TOGGLE_HIGHLIGHTS'
  | 'HIGHLIGHT_ELEMENT'
  | 'GET_SETTINGS'
  | 'UPDATE_SETTINGS'
  | 'GET_RESULTS'
  | 'CLEAR_RESULTS'
  | 'EXPORT_RESULTS'
  | 'AUTHENTICATE'
  | 'SYNC_DATA';

export interface Message<T = any> {
  type: MessageType;
  payload?: T;
  tabId?: number;
  timestamp: number;
}

export interface ScanPageMessage extends Message {
  type: 'SCAN_PAGE';
  payload: {
    url: string;
    config: ScannerConfig;
  };
}

export interface ScanCompleteMessage extends Message {
  type: 'SCAN_COMPLETE';
  payload: ScanResult;
}

export interface ScanErrorMessage extends Message {
  type: 'SCAN_ERROR';
  payload: {
    error: string;
    url: string;
  };
}

// ============================================================================
// Highlight Types
// ============================================================================

export interface HighlightStyle {
  color: string;
  borderWidth: number;
  borderStyle: string;
  backgroundColor: string;
  opacity: number;
  zIndex: number;
}

export interface HighlightConfig {
  critical: HighlightStyle;
  serious: HighlightStyle;
  moderate: HighlightStyle;
  minor: HighlightStyle;
}

// ============================================================================
// Accessibility Tree
// ============================================================================

export interface AccessibilityNode {
  id: string;
  role: string;
  name?: string;
  description?: string;
  value?: string;
  children: AccessibilityNode[];
  properties: Record<string, any>;
  states: string[];
  element: ElementInfo;
  issues: AccessibilityIssue[];
}

// ============================================================================
// API Types
// ============================================================================

export interface APIResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: number;
}

export interface AuthResponse {
  token: string;
  userId: string;
  expiresAt: number;
}

export interface SyncRequest {
  results: ScanResult[];
  settings: UserSettings;
  timestamp: number;
}

export interface SyncResponse {
  synced: boolean;
  conflicts: any[];
}

// ============================================================================
// Storage Types
// ============================================================================

export interface StorageData {
  settings: UserSettings;
  results: ScanResult[];
  auth: {
    token?: string;
    userId?: string;
    expiresAt?: number;
  };
  cache: {
    lastScan?: ScanResult;
    recentScans: ScanResult[];
  };
}

// ============================================================================
// DevTools Types
// ============================================================================

export interface DevToolsPanel {
  port: chrome.runtime.Port;
  tabId: number;
}

export interface InspectorState {
  selectedElement?: HTMLElement;
  selectedIssue?: AccessibilityIssue;
  highlightEnabled: boolean;
  treeExpanded: boolean;
}

// ============================================================================
// Badge Types
// ============================================================================

export interface BadgeState {
  text: string;
  color: string;
  title: string;
}

// ============================================================================
// Export Types
// ============================================================================

export type ExportFormat = 'json' | 'csv' | 'html' | 'pdf';

export interface ExportOptions {
  format: ExportFormat;
  includeDetails: boolean;
  includeSummary: boolean;
  includeScreenshots: boolean;
}

// ============================================================================
// Rule Engine Types
// ============================================================================

export interface Rule {
  id: string;
  name: string;
  description: string;
  category: IssueCategory;
  wcagCriteria: string[];
  wcagLevel: WCAGLevel;
  severity: SeverityLevel;
  check: (element: HTMLElement, context: ScanContext) => RuleResult | null;
  help: string;
}

export interface RuleResult {
  passed: boolean;
  message: string;
  suggestion: string;
  impact: string;
}

export interface ScanContext {
  document: Document;
  url: string;
  viewport: { width: number; height: number };
  colorContrast: boolean;
  keyboardNav: boolean;
}

// ============================================================================
// Utility Types
// ============================================================================

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export type RequireAtLeastOne<T, Keys extends keyof T = keyof T> = Pick<
  T,
  Exclude<keyof T, Keys>
> &
  {
    [K in Keys]-?: Required<Pick<T, K>> & Partial<Pick<T, Exclude<Keys, K>>>;
  }[Keys];
