/**
 * Accessibility Dashboard
 *
 * Main dashboard component displaying real-time accessibility metrics,
 * issue breakdowns, compliance status, and trend charts.
 */

import React, { useState, useMemo } from 'react';
import { useTheme } from '../enterprise/styles/theme';
import { Button } from '../enterprise/Button';
import { Tabs } from '../enterprise/Tabs';
import { useAccessibility, useIssueStats, useCompliance, useScanStatus, useAccessibilityScore } from './useAccessibility';
import { IssueLevel, ComplianceStandard, QuickAction } from './types';

interface AccessibilityDashboardProps {
  onNavigateToIssues?: () => void;
  onNavigateToReports?: () => void;
  onNavigateToSettings?: () => void;
}

export function AccessibilityDashboard({
  onNavigateToIssues,
  onNavigateToReports,
  onNavigateToSettings,
}: AccessibilityDashboardProps) {
  const { theme } = useTheme();
  const { loading, error, startScan, refreshData } = useAccessibility();
  const stats = useIssueStats();
  const scoreData = useAccessibilityScore();
  const { isScanning, lastScan } = useScanStatus();
  const [activeView, setActiveView] = useState<'overview' | 'compliance' | 'trends'>('overview');

  if (loading) {
    return (
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          height: '400px',
          color: theme.colors.text.secondary,
        }}
        role="status"
        aria-live="polite"
      >
        <div
          style={{
            width: '48px',
            height: '48px',
            border: `4px solid ${theme.colors.border.secondary}`,
            borderTopColor: theme.colors.interactive.primary,
            borderRadius: '50%',
            animation: 'spin 1s linear infinite',
          }}
          aria-hidden="true"
        />
        <p style={{ marginTop: theme.spacing[4] }}>Loading accessibility dashboard...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div
        style={{
          padding: theme.spacing[6],
          backgroundColor: theme.colors.status.error,
          color: theme.colors.text.inverse,
          borderRadius: theme.borderRadius.md,
        }}
        role="alert"
      >
        <h2 style={{ marginBottom: theme.spacing[2] }}>Error Loading Dashboard</h2>
        <p>{error.message}</p>
        <Button
          variant="secondary"
          onClick={() => refreshData()}
          style={{ marginTop: theme.spacing[4] }}
        >
          Retry
        </Button>
      </div>
    );
  }

  const quickActions: QuickAction[] = [
    {
      id: 'run-scan',
      label: isScanning ? 'Scanning...' : 'Run Scan',
      icon: '🔍',
      action: () => startScan(),
      disabled: isScanning,
      tooltip: 'Run accessibility scan on current application',
    },
    {
      id: 'view-issues',
      label: 'View Issues',
      icon: '📋',
      action: () => onNavigateToIssues?.(),
      tooltip: 'Browse all accessibility issues',
    },
    {
      id: 'generate-report',
      label: 'Generate Report',
      icon: '📄',
      action: () => onNavigateToReports?.(),
      tooltip: 'Create compliance report',
    },
    {
      id: 'settings',
      label: 'Settings',
      icon: '⚙️',
      action: () => onNavigateToSettings?.(),
      tooltip: 'Configure accessibility settings',
    },
  ];

  return (
    <div
      style={{
        padding: theme.spacing[6],
        backgroundColor: theme.colors.background.primary,
        minHeight: '100vh',
      }}
    >
      {/* Header */}
      <header
        style={{
          marginBottom: theme.spacing[6],
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <div>
          <h1
            style={{
              fontSize: theme.typography.fontSize['2xl'],
              fontWeight: theme.typography.fontWeight.bold,
              color: theme.colors.text.primary,
              margin: 0,
            }}
          >
            Accessibility Dashboard
          </h1>
          <p
            style={{
              fontSize: theme.typography.fontSize.sm,
              color: theme.colors.text.secondary,
              marginTop: theme.spacing[2],
            }}
          >
            Real-time accessibility monitoring and compliance tracking
          </p>
        </div>
        <div style={{ display: 'flex', gap: theme.spacing[2] }}>
          <Button
            variant="ghost"
            onClick={() => refreshData()}
            aria-label="Refresh dashboard data"
          >
            🔄 Refresh
          </Button>
        </div>
      </header>

      {/* Accessibility Score Card */}
      {scoreData && (
        <ScoreCard
          score={scoreData.overall}
          grade={scoreData.grade}
          color={scoreData.color}
          trend={scoreData.trend}
          change={scoreData.change}
          lastUpdated={scoreData.lastUpdated}
        />
      )}

      {/* Issue Summary */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
          gap: theme.spacing[4],
          marginTop: theme.spacing[6],
        }}
      >
        <StatCard
          title="Total Issues"
          value={stats.totalIssues}
          icon="🔍"
          color={theme.colors.status.info}
        />
        <StatCard
          title="Open Issues"
          value={stats.openIssues}
          icon="⚠️"
          color={theme.colors.status.warning}
        />
        <StatCard
          title="In Progress"
          value={stats.inProgressIssues}
          icon="🔧"
          color={theme.colors.interactive.primary}
        />
        <StatCard
          title="Fixed Issues"
          value={stats.fixedIssues}
          icon="✅"
          color={theme.colors.status.success}
        />
      </div>

      {/* Issue Breakdown by Severity */}
      <div style={{ marginTop: theme.spacing[6] }}>
        <h2
          style={{
            fontSize: theme.typography.fontSize.xl,
            fontWeight: theme.typography.fontWeight.semibold,
            color: theme.colors.text.primary,
            marginBottom: theme.spacing[4],
          }}
        >
          Issues by Severity
        </h2>
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
            gap: theme.spacing[3],
          }}
        >
          <SeverityCard
            level={IssueLevel.Critical}
            count={stats.byLevel[IssueLevel.Critical] || 0}
            color="#dc2626"
          />
          <SeverityCard
            level={IssueLevel.Serious}
            count={stats.byLevel[IssueLevel.Serious] || 0}
            color="#ea580c"
          />
          <SeverityCard
            level={IssueLevel.Moderate}
            count={stats.byLevel[IssueLevel.Moderate] || 0}
            color="#f59e0b"
          />
          <SeverityCard
            level={IssueLevel.Minor}
            count={stats.byLevel[IssueLevel.Minor] || 0}
            color="#10b981"
          />
        </div>
      </div>

      {/* Quick Actions */}
      <div style={{ marginTop: theme.spacing[6] }}>
        <h2
          style={{
            fontSize: theme.typography.fontSize.xl,
            fontWeight: theme.typography.fontWeight.semibold,
            color: theme.colors.text.primary,
            marginBottom: theme.spacing[4],
          }}
        >
          Quick Actions
        </h2>
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
            gap: theme.spacing[3],
          }}
        >
          {quickActions.map((action) => (
            <Button
              key={action.id}
              variant="secondary"
              size="lg"
              fullWidth
              onClick={action.action}
              disabled={action.disabled}
              title={action.tooltip}
              style={{
                justifyContent: 'flex-start',
                padding: theme.spacing[4],
              }}
            >
              <span style={{ fontSize: '24px', marginRight: theme.spacing[2] }}>
                {action.icon}
              </span>
              {action.label}
            </Button>
          ))}
        </div>
      </div>

      {/* Tabs for detailed views */}
      <div style={{ marginTop: theme.spacing[8] }}>
        <Tabs
          tabs={[
            {
              id: 'overview',
              label: 'Overview',
              content: <OverviewTab stats={stats} />,
            },
            {
              id: 'compliance',
              label: 'Compliance',
              content: <ComplianceTab />,
            },
            {
              id: 'trends',
              label: 'Trends',
              content: <TrendsTab />,
            },
          ]}
          activeTab={activeView}
          onChange={(id) => setActiveView(id as any)}
          variant="line"
        />
      </div>

      {/* Last Scan Info */}
      {lastScan && (
        <div
          style={{
            marginTop: theme.spacing[6],
            padding: theme.spacing[4],
            backgroundColor: theme.colors.background.secondary,
            borderRadius: theme.borderRadius.md,
            fontSize: theme.typography.fontSize.sm,
            color: theme.colors.text.secondary,
          }}
        >
          <strong>Last Scan:</strong> {new Date(lastScan.timestamp).toLocaleString()} |{' '}
          <strong>Duration:</strong> {lastScan.duration}ms |{' '}
          <strong>Pages Scanned:</strong> {lastScan.pagesScanned}
        </div>
      )}
    </div>
  );
}

// Score Card Component
function ScoreCard({
  score,
  grade,
  color,
  trend,
  change,
  lastUpdated,
}: {
  score: number;
  grade: string;
  color: string;
  trend: 'improving' | 'declining' | 'stable';
  change: number;
  lastUpdated: Date;
}) {
  const { theme } = useTheme();

  const trendIcon = trend === 'improving' ? '📈' : trend === 'declining' ? '📉' : '➡️';
  const trendColor =
    trend === 'improving'
      ? theme.colors.status.success
      : trend === 'declining'
      ? theme.colors.status.error
      : theme.colors.text.secondary;

  const scoreColor =
    color === 'success'
      ? theme.colors.status.success
      : color === 'warning'
      ? theme.colors.status.warning
      : theme.colors.status.error;

  return (
    <div
      style={{
        padding: theme.spacing[6],
        backgroundColor: theme.colors.background.secondary,
        borderRadius: theme.borderRadius.lg,
        border: `3px solid ${scoreColor}`,
        textAlign: 'center',
      }}
      role="region"
      aria-label="Accessibility score"
    >
      <div
        style={{
          fontSize: '72px',
          fontWeight: theme.typography.fontWeight.bold,
          color: scoreColor,
          lineHeight: 1,
        }}
        aria-label={`Score: ${score} out of 100`}
      >
        {score}
      </div>
      <div
        style={{
          fontSize: '36px',
          fontWeight: theme.typography.fontWeight.semibold,
          color: scoreColor,
          marginTop: theme.spacing[2],
        }}
      >
        Grade {grade}
      </div>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          gap: theme.spacing[2],
          marginTop: theme.spacing[4],
          fontSize: theme.typography.fontSize.lg,
          color: trendColor,
        }}
      >
        <span aria-hidden="true">{trendIcon}</span>
        <span>
          {trend === 'stable' ? 'No change' : `${change > 0 ? '+' : ''}${change.toFixed(1)}%`}
        </span>
      </div>
      <div
        style={{
          marginTop: theme.spacing[4],
          fontSize: theme.typography.fontSize.sm,
          color: theme.colors.text.tertiary,
        }}
      >
        Last updated: {new Date(lastUpdated).toLocaleString()}
      </div>
    </div>
  );
}

// Stat Card Component
function StatCard({
  title,
  value,
  icon,
  color,
}: {
  title: string;
  value: number;
  icon: string;
  color: string;
}) {
  const { theme } = useTheme();

  return (
    <div
      style={{
        padding: theme.spacing[5],
        backgroundColor: theme.colors.background.secondary,
        borderRadius: theme.borderRadius.md,
        borderLeft: `4px solid ${color}`,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: theme.spacing[3] }}>
        <div
          style={{
            fontSize: '32px',
            width: '48px',
            height: '48px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: theme.colors.background.primary,
            borderRadius: theme.borderRadius.md,
          }}
          aria-hidden="true"
        >
          {icon}
        </div>
        <div>
          <div
            style={{
              fontSize: theme.typography.fontSize.sm,
              color: theme.colors.text.secondary,
              marginBottom: theme.spacing[1],
            }}
          >
            {title}
          </div>
          <div
            style={{
              fontSize: theme.typography.fontSize['2xl'],
              fontWeight: theme.typography.fontWeight.bold,
              color: theme.colors.text.primary,
            }}
          >
            {value}
          </div>
        </div>
      </div>
    </div>
  );
}

// Severity Card Component
function SeverityCard({
  level,
  count,
  color,
}: {
  level: IssueLevel;
  count: number;
  color: string;
}) {
  const { theme } = useTheme();

  const levelLabels = {
    [IssueLevel.Critical]: 'Critical',
    [IssueLevel.Serious]: 'Serious',
    [IssueLevel.Moderate]: 'Moderate',
    [IssueLevel.Minor]: 'Minor',
  };

  return (
    <div
      style={{
        padding: theme.spacing[4],
        backgroundColor: theme.colors.background.secondary,
        borderRadius: theme.borderRadius.md,
        border: `2px solid ${color}`,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
      }}
    >
      <div
        style={{
          width: '64px',
          height: '64px',
          borderRadius: '50%',
          backgroundColor: color,
          color: '#ffffff',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: '28px',
          fontWeight: theme.typography.fontWeight.bold,
          marginBottom: theme.spacing[3],
        }}
      >
        {count}
      </div>
      <div
        style={{
          fontSize: theme.typography.fontSize.base,
          fontWeight: theme.typography.fontWeight.semibold,
          color: theme.colors.text.primary,
        }}
      >
        {levelLabels[level]}
      </div>
    </div>
  );
}

// Overview Tab
function OverviewTab({ stats }: any) {
  const { theme } = useTheme();
  const { issues } = useAccessibility();

  const recentIssues = useMemo(
    () => issues.slice(0, 5).sort((a, b) =>
      new Date(b.detectedAt).getTime() - new Date(a.detectedAt).getTime()
    ),
    [issues]
  );

  return (
    <div style={{ padding: theme.spacing[4] }}>
      <h3
        style={{
          fontSize: theme.typography.fontSize.lg,
          fontWeight: theme.typography.fontWeight.semibold,
          marginBottom: theme.spacing[4],
        }}
      >
        Recent Issues
      </h3>
      <div style={{ display: 'flex', flexDirection: 'column', gap: theme.spacing[3] }}>
        {recentIssues.length === 0 ? (
          <p style={{ color: theme.colors.text.secondary }}>No recent issues found</p>
        ) : (
          recentIssues.map((issue) => (
            <div
              key={issue.id}
              style={{
                padding: theme.spacing[3],
                backgroundColor: theme.colors.background.secondary,
                borderRadius: theme.borderRadius.md,
                borderLeft: `4px solid ${
                  issue.level === IssueLevel.Critical
                    ? '#dc2626'
                    : issue.level === IssueLevel.Serious
                    ? '#ea580c'
                    : issue.level === IssueLevel.Moderate
                    ? '#f59e0b'
                    : '#10b981'
                }`,
              }}
            >
              <div
                style={{
                  fontSize: theme.typography.fontSize.base,
                  fontWeight: theme.typography.fontWeight.medium,
                  color: theme.colors.text.primary,
                }}
              >
                {issue.title}
              </div>
              <div
                style={{
                  fontSize: theme.typography.fontSize.sm,
                  color: theme.colors.text.secondary,
                  marginTop: theme.spacing[1],
                }}
              >
                {issue.category} • {new Date(issue.detectedAt).toLocaleDateString()}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

// Compliance Tab
function ComplianceTab() {
  const { theme } = useTheme();
  const complianceStatus = useCompliance();

  return (
    <div style={{ padding: theme.spacing[4] }}>
      <h3
        style={{
          fontSize: theme.typography.fontSize.lg,
          fontWeight: theme.typography.fontWeight.semibold,
          marginBottom: theme.spacing[4],
        }}
      >
        Compliance Status
      </h3>
      <div style={{ display: 'flex', flexDirection: 'column', gap: theme.spacing[3] }}>
        {Array.isArray(complianceStatus) && complianceStatus.length > 0 ? (
          complianceStatus.map((status) => (
            <div
              key={status.standard}
              style={{
                padding: theme.spacing[4],
                backgroundColor: theme.colors.background.secondary,
                borderRadius: theme.borderRadius.md,
                border: `2px solid ${
                  status.passed ? theme.colors.status.success : theme.colors.status.error
                }`,
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <div>
                  <div
                    style={{
                      fontSize: theme.typography.fontSize.base,
                      fontWeight: theme.typography.fontWeight.semibold,
                      color: theme.colors.text.primary,
                    }}
                  >
                    {status.standard}
                  </div>
                  <div
                    style={{
                      fontSize: theme.typography.fontSize.sm,
                      color: theme.colors.text.secondary,
                      marginTop: theme.spacing[1],
                    }}
                  >
                    {status.passedCriteria} / {status.totalCriteria} criteria passed
                  </div>
                </div>
                <div
                  style={{
                    fontSize: theme.typography.fontSize['2xl'],
                    fontWeight: theme.typography.fontWeight.bold,
                    color: status.passed ? theme.colors.status.success : theme.colors.status.error,
                  }}
                >
                  {status.percentage.toFixed(0)}%
                </div>
              </div>
            </div>
          ))
        ) : (
          <p style={{ color: theme.colors.text.secondary }}>No compliance data available</p>
        )}
      </div>
    </div>
  );
}

// Trends Tab
function TrendsTab() {
  const { theme } = useTheme();

  return (
    <div style={{ padding: theme.spacing[4] }}>
      <h3
        style={{
          fontSize: theme.typography.fontSize.lg,
          fontWeight: theme.typography.fontWeight.semibold,
          marginBottom: theme.spacing[4],
        }}
      >
        Accessibility Trends
      </h3>
      <p style={{ color: theme.colors.text.secondary }}>
        Trend charts will be displayed here showing improvement over time.
      </p>
    </div>
  );
}
