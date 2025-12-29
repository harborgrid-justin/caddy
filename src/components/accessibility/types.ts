/**
 * Accessibility Type Definitions
 *
 * TypeScript interfaces for accessibility dashboard components and features.
 */

export enum IssueLevel {
  Critical = 'critical',
  Serious = 'serious',
  Moderate = 'moderate',
  Minor = 'minor',
}

export enum IssueCategory {
  ColorContrast = 'color-contrast',
  KeyboardNavigation = 'keyboard-navigation',
  ScreenReader = 'screen-reader',
  FocusManagement = 'focus-management',
  SemanticHTML = 'semantic-html',
  AriaLabels = 'aria-labels',
  FormAccessibility = 'form-accessibility',
  ImageAltText = 'image-alt-text',
  HeadingStructure = 'heading-structure',
  LinkPurpose = 'link-purpose',
  TableAccessibility = 'table-accessibility',
  MediaCaptions = 'media-captions',
  TimingControls = 'timing-controls',
  ErrorIdentification = 'error-identification',
}

export enum ComplianceStandard {
  WCAG_2_0_A = 'WCAG 2.0 Level A',
  WCAG_2_0_AA = 'WCAG 2.0 Level AA',
  WCAG_2_0_AAA = 'WCAG 2.0 Level AAA',
  WCAG_2_1_A = 'WCAG 2.1 Level A',
  WCAG_2_1_AA = 'WCAG 2.1 Level AA',
  WCAG_2_1_AAA = 'WCAG 2.1 Level AAA',
  WCAG_2_2_A = 'WCAG 2.2 Level A',
  WCAG_2_2_AA = 'WCAG 2.2 Level AA',
  WCAG_2_2_AAA = 'WCAG 2.2 Level AAA',
  Section508 = 'Section 508',
  ADA = 'ADA',
}

export enum ReportFormat {
  PDF = 'pdf',
  CSV = 'csv',
  JSON = 'json',
  HTML = 'html',
  XLSX = 'xlsx',
}

export enum ScanStatus {
  Idle = 'idle',
  Running = 'running',
  Completed = 'completed',
  Failed = 'failed',
}

export interface AccessibilityIssue {
  id: string;
  level: IssueLevel;
  category: IssueCategory;
  title: string;
  description: string;
  wcagCriteria: string[];
  element?: {
    selector: string;
    html: string;
    xpath: string;
    boundingBox?: {
      x: number;
      y: number;
      width: number;
      height: number;
    };
  };
  suggestedFix?: string;
  codeSnippet?: string;
  fixedCodeSnippet?: string;
  references: string[];
  impact: string;
  affectedUsers: string[];
  detectedAt: Date;
  fixedAt?: Date;
  status: 'open' | 'in-progress' | 'fixed' | 'wont-fix' | 'false-positive';
  assignee?: string;
  notes?: string;
  pageUrl?: string;
  componentName?: string;
}

export interface AccessibilityScore {
  overall: number;
  byCategory: Record<IssueCategory, number>;
  byLevel: Record<IssueLevel, number>;
  trend: 'improving' | 'declining' | 'stable';
  change: number;
  lastUpdated: Date;
}

export interface ComplianceStatus {
  standard: ComplianceStandard;
  passed: boolean;
  passedCriteria: number;
  totalCriteria: number;
  percentage: number;
  failedCriteria: Array<{
    criteria: string;
    description: string;
    issues: string[];
  }>;
  lastChecked: Date;
}

export interface ScanResult {
  id: string;
  timestamp: Date;
  status: ScanStatus;
  duration: number;
  pagesScanned: number;
  issuesFound: number;
  issuesByLevel: Record<IssueLevel, number>;
  issuesByCategory: Record<IssueCategory, number>;
  score: AccessibilityScore;
  complianceStatus: ComplianceStatus[];
  error?: string;
}

export interface ScanConfig {
  enabled: boolean;
  autoScan: boolean;
  scanInterval: number;
  includePatterns: string[];
  excludePatterns: string[];
  standards: ComplianceStandard[];
  checkColorContrast: boolean;
  checkKeyboardNav: boolean;
  checkScreenReader: boolean;
  checkAriaLabels: boolean;
  checkSemanticHTML: boolean;
  maxIssuesPerLevel: Record<IssueLevel, number>;
  webhookUrl?: string;
  notifyOnNewIssues: boolean;
}

export interface TrendDataPoint {
  timestamp: Date;
  score: number;
  issuesCount: number;
  issuesByLevel: Record<IssueLevel, number>;
}

export interface AccessibilityTrend {
  period: '24h' | '7d' | '30d' | '90d' | '1y';
  dataPoints: TrendDataPoint[];
  averageScore: number;
  scoreImprovement: number;
  issuesFixed: number;
  issuesCreated: number;
  netChange: number;
}

export interface IssueFilter {
  levels?: IssueLevel[];
  categories?: IssueCategory[];
  status?: Array<'open' | 'in-progress' | 'fixed' | 'wont-fix' | 'false-positive'>;
  standards?: ComplianceStandard[];
  dateRange?: {
    start: Date;
    end: Date;
  };
  searchQuery?: string;
  assignee?: string;
  componentName?: string;
  pageUrl?: string;
}

export interface IssueSortConfig {
  field: keyof AccessibilityIssue;
  direction: 'asc' | 'desc';
}

export interface ReportConfig {
  title: string;
  format: ReportFormat;
  includeExecutiveSummary: boolean;
  includeTechnicalDetails: boolean;
  includeCodeSnippets: boolean;
  includeScreenshots: boolean;
  includeRecommendations: boolean;
  includeCompliance: boolean;
  includeTrends: boolean;
  standards: ComplianceStandard[];
  dateRange?: {
    start: Date;
    end: Date;
  };
  recipients?: string[];
  schedule?: {
    enabled: boolean;
    frequency: 'daily' | 'weekly' | 'monthly';
    dayOfWeek?: number;
    dayOfMonth?: number;
    time: string;
  };
}

export interface AccessibilitySettings {
  tenantId?: string;
  scanConfig: ScanConfig;
  notifications: {
    email: boolean;
    slack: boolean;
    webhook: boolean;
    emailAddresses: string[];
    slackChannel?: string;
    webhookUrl?: string;
  };
  integrations: {
    jira?: {
      enabled: boolean;
      url: string;
      project: string;
      apiToken: string;
      autoCreateIssues: boolean;
    };
    github?: {
      enabled: boolean;
      repo: string;
      token: string;
      autoCreateIssues: boolean;
    };
    ci?: {
      enabled: boolean;
      failOnCritical: boolean;
      failOnSerious: boolean;
      minScore: number;
    };
  };
  team: {
    members: Array<{
      id: string;
      name: string;
      email: string;
      role: 'admin' | 'developer' | 'viewer';
    }>;
    defaultAssignee?: string;
  };
}

export interface QuickAction {
  id: string;
  label: string;
  icon: string;
  action: () => void;
  disabled?: boolean;
  tooltip?: string;
}

export interface ExportOptions {
  format: ReportFormat;
  filename: string;
  includeAttachments: boolean;
}

export interface AccessibilityContextValue {
  // State
  issues: AccessibilityIssue[];
  score: AccessibilityScore | null;
  complianceStatus: ComplianceStatus[];
  trends: AccessibilityTrend[];
  currentScan: ScanResult | null;
  scanHistory: ScanResult[];
  settings: AccessibilitySettings | null;
  loading: boolean;
  error: Error | null;

  // Scanning
  startScan: (config?: Partial<ScanConfig>) => Promise<void>;
  stopScan: () => Promise<void>;
  getScanResult: (scanId: string) => Promise<ScanResult>;

  // Issue Management
  getIssues: (filter?: IssueFilter, sort?: IssueSortConfig) => Promise<AccessibilityIssue[]>;
  getIssueById: (issueId: string) => Promise<AccessibilityIssue>;
  updateIssue: (issueId: string, updates: Partial<AccessibilityIssue>) => Promise<void>;
  deleteIssue: (issueId: string) => Promise<void>;
  bulkUpdateIssues: (issueIds: string[], updates: Partial<AccessibilityIssue>) => Promise<void>;
  markIssueAsFixed: (issueId: string) => Promise<void>;
  markIssueAsFalsePositive: (issueId: string, reason: string) => Promise<void>;

  // Compliance
  checkCompliance: (standard: ComplianceStandard) => Promise<ComplianceStatus>;
  getComplianceReport: (standards: ComplianceStandard[]) => Promise<any>;

  // Trends & Analytics
  getTrends: (period: '24h' | '7d' | '30d' | '90d' | '1y') => Promise<AccessibilityTrend>;
  getScoreHistory: (days: number) => Promise<TrendDataPoint[]>;

  // Settings
  updateSettings: (updates: Partial<AccessibilitySettings>) => Promise<void>;
  resetSettings: () => Promise<void>;

  // Reporting
  generateReport: (config: ReportConfig) => Promise<Blob>;
  scheduleReport: (config: ReportConfig) => Promise<void>;
  exportData: (options: ExportOptions) => Promise<Blob>;

  // Element Highlighting
  highlightElement: (selector: string) => void;
  unhighlightAll: () => void;

  // Refresh
  refreshData: () => Promise<void>;
}
