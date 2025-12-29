/**
 * CADDY v0.3.0 - Master Component Export
 *
 * This file provides a unified import path for all CADDY UI components.
 * Instead of importing from individual component directories, you can
 * import everything from this single entry point.
 *
 * @example
 * ```typescript
 * import {
 *   AccessibilityDashboard,
 *   LoginForm,
 *   ScheduleManager,
 *   Button,
 *   Table
 * } from '@/components';
 * ```
 *
 * ## Component Categories
 *
 * ### Enterprise UI Components (v0.2.5)
 * - Core UI components: Button, Input, Select, Modal, Tooltip
 * - Data display: Table, Tree, Tabs
 * - CAD-specific: PropertyPanel, Toolbar, StatusBar, ColorPicker
 * - Layout: Splitter, ContextMenu
 * - Theming: ThemeProvider, useTheme
 *
 * ### Accessibility Components (v0.3.0)
 * - AccessibilityDashboard: Main dashboard for accessibility features
 * - AccessibilityProvider: Context provider for accessibility state
 * - ComplianceReport: WCAG compliance reporting
 * - IssueExplorer: Issue browsing and remediation
 * - useAccessibility: Hook for accessibility features
 *
 * ### Authentication Components (v0.3.0)
 * - LoginForm: User authentication form
 * - MFASetup: Multi-factor authentication setup
 * - RoleManager: RBAC role management UI
 * - SessionMonitor: Active session monitoring
 * - SSOConfig: SSO provider configuration
 *
 * ### Scheduling Components (v0.3.0)
 * - ScheduleManager: Job scheduling interface
 * - MonitoringDashboard: Job monitoring and health
 * - NotificationSettings: Alert configuration
 *
 * ### Integration Components (v0.3.0)
 * - IntegrationHub: CI/CD integration management
 * - GitHubSetup: GitHub Actions configuration
 * - CIConfigGenerator: CI configuration generator
 *
 * ### Team Collaboration Components (v0.3.0)
 * - WorkspaceManager: Team workspace management
 * - MemberList: Team member directory
 * - AssignmentBoard: Task assignment interface
 * - ActivityFeed: Team activity stream
 *
 * ### AI/ML Components (v0.3.0)
 * - AIAssistant: AI-powered assistance
 * - SuggestionPanel: AI suggestions and recommendations
 *
 * ### Analytics Components (v0.3.0)
 * - Dashboard: Analytics overview
 * - Charts: Data visualization components
 * - Reports: Report generation and viewing
 */

// ============================================================================
// Enterprise UI Components (v0.2.5 - Core Component Library)
// ============================================================================

export {
  // Basic Components
  Button,
  Input,
  Select,
  Modal,
  Tooltip,

  // Data Display
  Table,
  Tree,
  Tabs,

  // Interactive Components
  ContextMenu,
  Splitter,
  ColorPicker,

  // CAD-Specific Components
  PropertyPanel,
  Toolbar,
  StatusBar,

  // Theme System
  ThemeProvider,
  useTheme,

  // Design System
  tokens,
  animations,

  // Types
  type ButtonProps,
  type InputProps,
  type SelectProps,
  type ModalProps,
  type TooltipProps,
  type TableProps,
  type TreeProps,
  type TabsProps,
  type ContextMenuProps,
  type SplitterProps,
  type ColorPickerProps,
  type PropertyPanelProps,
  type ToolbarProps,
  type StatusBarProps,
  type Theme,
  type ThemeMode,
} from './enterprise';

// ============================================================================
// Accessibility Components (v0.3.0)
// ============================================================================

export {
  // Main Components
  AccessibilityDashboard,
  AccessibilityProvider,
  ComplianceReport,
  IssueExplorer,

  // Hooks
  useAccessibility,

  // Types
  type AccessibilityState,
  type AccessibilityIssue,
  type ComplianceLevel,
  type ScanResult,
  type RemediationSuggestion,
} from './accessibility';

// ============================================================================
// Authentication Components (v0.3.0)
// ============================================================================

export {
  // Authentication UI
  LoginForm,
  MFASetup,
  RoleManager,
  SessionMonitor,
  SSOConfig,

  // Types
  type AuthState,
  type User,
  type Role,
  type Permission,
  type MFAMethod,
  type SSOProvider,
} from './auth';

// ============================================================================
// Scheduling Components (v0.3.0)
// ============================================================================

export {
  // Scheduling UI
  ScheduleManager,
  MonitoringDashboard,
  NotificationSettings,

  // Types
  type Schedule,
  type Job,
  type JobStatus,
  type MonitoringMetrics,
  type NotificationChannel,
} from './scheduling';

// ============================================================================
// Integration Components (v0.3.0)
// ============================================================================

export {
  // Integration UI
  IntegrationHub,
  GitHubSetup,
  CIConfigGenerator,

  // Types
  type Integration,
  type IntegrationType,
  type CIProvider,
  type WebhookConfig,
} from './integrations';

// ============================================================================
// Team Collaboration Components (v0.3.0)
// ============================================================================

export {
  // Team UI
  WorkspaceManager,
  MemberList,
  AssignmentBoard,
  ActivityFeed,

  // Types
  type Workspace,
  type TeamMember,
  type Assignment,
  type Activity,
  type MemberRole,
} from './teams';

// ============================================================================
// AI/ML Components (v0.3.0)
// ============================================================================

export {
  // AI UI
  AIAssistant,
  SuggestionPanel,

  // Types
  type AISuggestion,
  type ConfidenceScore,
  type ModelVersion,
} from './ai';

// ============================================================================
// Analytics Components (v0.3.0)
// ============================================================================

export {
  // Analytics UI
  Dashboard as AnalyticsDashboard,
  Charts,
  Reports,

  // Types
  type Metric,
  type ChartData,
  type Report,
  type AggregationPeriod,
} from './analytics';

// ============================================================================
// Utility Exports
// ============================================================================

/**
 * Component version information
 */
export const COMPONENT_VERSION = '0.3.0';

/**
 * List of all available component categories
 */
export const COMPONENT_CATEGORIES = [
  'enterprise',
  'accessibility',
  'auth',
  'scheduling',
  'integrations',
  'teams',
  'ai',
  'analytics',
] as const;

/**
 * Type for component categories
 */
export type ComponentCategory = typeof COMPONENT_CATEGORIES[number];

/**
 * Get all components in a specific category
 *
 * @param category - The component category
 * @returns Description of components in the category
 */
export function getComponentsByCategory(category: ComponentCategory): string[] {
  const categories: Record<ComponentCategory, string[]> = {
    enterprise: [
      'Button', 'Input', 'Select', 'Modal', 'Tooltip',
      'Table', 'Tree', 'Tabs', 'ContextMenu', 'Splitter',
      'ColorPicker', 'PropertyPanel', 'Toolbar', 'StatusBar'
    ],
    accessibility: [
      'AccessibilityDashboard', 'AccessibilityProvider',
      'ComplianceReport', 'IssueExplorer'
    ],
    auth: [
      'LoginForm', 'MFASetup', 'RoleManager',
      'SessionMonitor', 'SSOConfig'
    ],
    scheduling: [
      'ScheduleManager', 'MonitoringDashboard', 'NotificationSettings'
    ],
    integrations: [
      'IntegrationHub', 'GitHubSetup', 'CIConfigGenerator'
    ],
    teams: [
      'WorkspaceManager', 'MemberList', 'AssignmentBoard', 'ActivityFeed'
    ],
    ai: [
      'AIAssistant', 'SuggestionPanel'
    ],
    analytics: [
      'AnalyticsDashboard', 'Charts', 'Reports'
    ],
  };

  return categories[category] || [];
}

/**
 * Component registry for documentation and debugging
 */
export const COMPONENT_REGISTRY = {
  version: COMPONENT_VERSION,
  categories: COMPONENT_CATEGORIES,
  componentCount: Object.values(COMPONENT_CATEGORIES).reduce(
    (count, category) => count + getComponentsByCategory(category).length,
    0
  ),
  description: 'CADDY Enterprise Accessibility SaaS Component Library',
};

// ============================================================================
// Type Guards and Utilities
// ============================================================================

/**
 * Check if a string is a valid component category
 */
export function isComponentCategory(value: string): value is ComponentCategory {
  return COMPONENT_CATEGORIES.includes(value as ComponentCategory);
}

/**
 * Get component count by category
 */
export function getComponentCount(category?: ComponentCategory): number {
  if (category) {
    return getComponentsByCategory(category).length;
  }
  return COMPONENT_REGISTRY.componentCount;
}

// ============================================================================
// Development Utilities
// ============================================================================

/**
 * Log component library information (development only)
 */
export function logComponentInfo(): void {
  if (process.env.NODE_ENV === 'development') {
    console.group('CADDY Component Library v' + COMPONENT_VERSION);
    console.log('Total Components:', COMPONENT_REGISTRY.componentCount);
    console.log('Categories:', COMPONENT_CATEGORIES);

    COMPONENT_CATEGORIES.forEach(category => {
      console.log(
        `  ${category}:`,
        getComponentsByCategory(category).join(', ')
      );
    });

    console.groupEnd();
  }
}

// ============================================================================
// Default Export
// ============================================================================

/**
 * Default export provides component registry information
 */
export default COMPONENT_REGISTRY;
