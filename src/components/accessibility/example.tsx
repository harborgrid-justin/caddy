/**
 * Accessibility Dashboard - Example Usage
 *
 * This file demonstrates how to integrate and use the accessibility
 * dashboard components in your CADDY application.
 */

import React, { useState } from 'react';
import {
  AccessibilityProvider,
  AccessibilityDashboard,
  IssueExplorer,
  ComplianceReport,
  useAccessibility,
  IssueLevel,
  ComplianceStandard,
  ReportFormat,
} from './index';
import { Tabs } from '../enterprise/Tabs';
import './accessibility.css';

/**
 * Example 1: Complete Accessibility Dashboard Application
 */
export function AccessibilityApp() {
  const [activeTab, setActiveTab] = useState('dashboard');

  return (
    <AccessibilityProvider
      tenantId="example-tenant"
      apiBaseUrl="/api/accessibility"
    >
      <div style={{ padding: '24px' }}>
        <Tabs
          tabs={[
            {
              id: 'dashboard',
              label: 'Dashboard',
              icon: '📊',
              content: (
                <AccessibilityDashboard
                  onNavigateToIssues={() => setActiveTab('issues')}
                  onNavigateToReports={() => setActiveTab('reports')}
                  onNavigateToSettings={() => setActiveTab('settings')}
                />
              ),
            },
            {
              id: 'issues',
              label: 'Issues',
              icon: '🔍',
              badge: '12',
              content: (
                <IssueExplorer
                  showBulkActions={true}
                  onIssueClick={(issue) => {
                    console.log('Issue selected:', issue);
                  }}
                />
              ),
            },
            {
              id: 'reports',
              label: 'Reports',
              icon: '📄',
              content: (
                <ComplianceReport
                  defaultStandards={[
                    ComplianceStandard.WCAG_2_1_AA,
                    ComplianceStandard.Section508,
                    ComplianceStandard.ADA,
                  ]}
                  onExport={(format, blob) => {
                    console.log(`Report exported as ${format}`, blob);
                  }}
                />
              ),
            },
          ]}
          activeTab={activeTab}
          onChange={setActiveTab}
          variant="enclosed"
        />
      </div>
    </AccessibilityProvider>
  );
}

/**
 * Example 2: Critical Issues Dashboard
 * Shows only critical and serious issues
 */
export function CriticalIssuesDashboard() {
  return (
    <AccessibilityProvider>
      <IssueExplorer
        initialFilter={{
          levels: [IssueLevel.Critical, IssueLevel.Serious],
          status: ['open', 'in-progress'],
        }}
        showBulkActions={true}
      />
    </AccessibilityProvider>
  );
}

/**
 * Example 3: Executive Report Generator
 * Simplified interface for generating executive reports
 */
export function ExecutiveReportGenerator() {
  return (
    <AccessibilityProvider>
      <ComplianceReport
        defaultStandards={[ComplianceStandard.WCAG_2_1_AA]}
        onExport={(format, blob) => {
          // Send to email or cloud storage
          console.log('Report ready for executive review');
        }}
      />
    </AccessibilityProvider>
  );
}

/**
 * Example 4: Real-time Scanning Component
 * Automatically scans and displays results
 */
export function RealtimeScanner() {
  const { startScan, currentScan, issues } = useAccessibility();
  const [autoScan, setAutoScan] = useState(false);

  React.useEffect(() => {
    if (autoScan) {
      const interval = setInterval(() => {
        startScan();
      }, 300000); // Scan every 5 minutes

      return () => clearInterval(interval);
    }
  }, [autoScan, startScan]);

  return (
    <div>
      <h2>Real-time Accessibility Scanner</h2>
      <button onClick={() => setAutoScan(!autoScan)}>
        {autoScan ? 'Stop Auto-Scan' : 'Start Auto-Scan'}
      </button>

      {currentScan && (
        <div>
          <p>Scan Status: {currentScan.status}</p>
          <p>Issues Found: {currentScan.issuesFound}</p>
        </div>
      )}

      <ul>
        {issues.slice(0, 5).map((issue) => (
          <li key={issue.id}>
            [{issue.level}] {issue.title}
          </li>
        ))}
      </ul>
    </div>
  );
}

/**
 * Example 5: Compliance Status Widget
 * Small widget showing compliance status
 */
export function ComplianceStatusWidget() {
  const { complianceStatus } = useAccessibility();

  return (
    <div style={{
      padding: '16px',
      backgroundColor: '#f3f4f6',
      borderRadius: '8px',
    }}>
      <h3>Compliance Status</h3>
      {Array.isArray(complianceStatus) && complianceStatus.map((status) => (
        <div key={status.standard} style={{
          display: 'flex',
          justifyContent: 'space-between',
          padding: '8px 0',
          borderBottom: '1px solid #e5e7eb',
        }}>
          <span>{status.standard}</span>
          <span style={{
            color: status.passed ? '#10b981' : '#ef4444',
            fontWeight: 'bold',
          }}>
            {status.percentage.toFixed(0)}%
          </span>
        </div>
      ))}
    </div>
  );
}

/**
 * Example 6: Custom Hook Usage
 * Using accessibility hooks in custom components
 */
export function CustomAccessibilityComponent() {
  const {
    issues,
    score,
    startScan,
    updateIssue,
    markIssueAsFixed,
  } = useAccessibility();

  const handleFixIssue = async (issueId: string) => {
    await markIssueAsFixed(issueId);
    console.log('Issue marked as fixed!');
  };

  return (
    <div>
      <h2>Accessibility Score: {score?.overall || 0}/100</h2>
      <button onClick={() => startScan()}>Run Scan</button>

      <ul>
        {issues.filter(i => i.status === 'open').map((issue) => (
          <li key={issue.id}>
            <strong>[{issue.level}]</strong> {issue.title}
            <button onClick={() => handleFixIssue(issue.id)}>
              Mark as Fixed
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

/**
 * Example 7: Multi-Tenant Usage
 * Different tenants with isolated data
 */
export function MultiTenantAccessibility({ tenantId }: { tenantId: string }) {
  return (
    <AccessibilityProvider tenantId={tenantId}>
      <AccessibilityDashboard />
    </AccessibilityProvider>
  );
}

/**
 * Example 8: Filtered Issue List
 * Show specific categories of issues
 */
export function ColorContrastIssues() {
  return (
    <AccessibilityProvider>
      <IssueExplorer
        initialFilter={{
          categories: ['color-contrast' as any],
        }}
        showBulkActions={false}
      />
    </AccessibilityProvider>
  );
}

/**
 * Example 9: Scheduled Report Configuration
 * Set up automated weekly reports
 */
export function ScheduledReportsSetup() {
  return (
    <AccessibilityProvider>
      <ComplianceReport
        defaultStandards={[
          ComplianceStandard.WCAG_2_1_AA,
          ComplianceStandard.WCAG_2_2_AA,
        ]}
        onExport={(format, blob) => {
          console.log('Scheduled report generated');
        }}
      />
    </AccessibilityProvider>
  );
}

/**
 * Example 10: Integration with Existing Dashboard
 * Add accessibility panel to existing admin dashboard
 */
export function AdminDashboardWithAccessibility() {
  return (
    <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: '24px' }}>
      {/* Existing dashboard content */}
      <div>
        <h2>Main Dashboard</h2>
        {/* Your existing dashboard components */}
      </div>

      {/* Accessibility panel */}
      <aside>
        <AccessibilityProvider>
          <ComplianceStatusWidget />
        </AccessibilityProvider>
      </aside>
    </div>
  );
}

/**
 * Usage in Routes
 */
export const accessibilityRoutes = [
  {
    path: '/accessibility',
    element: <AccessibilityApp />,
  },
  {
    path: '/accessibility/issues',
    element: <CriticalIssuesDashboard />,
  },
  {
    path: '/accessibility/reports',
    element: <ExecutiveReportGenerator />,
  },
];

/**
 * Quick Start Example
 */
export function QuickStart() {
  return (
    <AccessibilityProvider>
      <AccessibilityDashboard />
    </AccessibilityProvider>
  );
}

export default AccessibilityApp;
