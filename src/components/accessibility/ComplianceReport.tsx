/**
 * Compliance Report
 *
 * Comprehensive reporting component with executive summary, detailed technical
 * reports, and export to multiple formats (PDF, CSV, JSON).
 */

import React, { useState, useMemo } from 'react';
import { useTheme } from '../enterprise/styles/theme';
import { Button } from '../enterprise/Button';
import { Tabs } from '../enterprise/Tabs';
import { Input } from '../enterprise/Input';
import { Select } from '../enterprise/Select';
import {
  useAccessibility,
  useCompliance,
  useIssueStats,
  useAccessibilityScore,
} from './useAccessibility';
import {
  ReportFormat,
  ComplianceStandard,
  ReportConfig,
  IssueLevel,
} from './types';

interface ComplianceReportProps {
  defaultStandards?: ComplianceStandard[];
  onExport?: (format: ReportFormat, data: Blob) => void;
}

export function ComplianceReport({
  defaultStandards = [
    ComplianceStandard.WCAG_2_1_AA,
    ComplianceStandard.Section508,
    ComplianceStandard.ADA,
  ],
  onExport,
}: ComplianceReportProps) {
  const { theme } = useTheme();
  const { generateReport, scheduleReport, exportData, issues } = useAccessibility();
  const stats = useIssueStats();
  const scoreData = useAccessibilityScore();
  const complianceStatus = useCompliance();

  const [selectedStandards, setSelectedStandards] = useState<ComplianceStandard[]>(defaultStandards);
  const [reportFormat, setReportFormat] = useState<ReportFormat>(ReportFormat.PDF);
  const [reportTitle, setReportTitle] = useState('Accessibility Compliance Report');
  const [includeExecutiveSummary, setIncludeExecutiveSummary] = useState(true);
  const [includeTechnicalDetails, setIncludeTechnicalDetails] = useState(true);
  const [includeCodeSnippets, setIncludeCodeSnippets] = useState(true);
  const [includeRecommendations, setIncludeRecommendations] = useState(true);
  const [includeTrends, setIncludeTrends] = useState(true);
  const [isGenerating, setIsGenerating] = useState(false);

  // Schedule settings
  const [scheduleEnabled, setScheduleEnabled] = useState(false);
  const [scheduleFrequency, setScheduleFrequency] = useState<'daily' | 'weekly' | 'monthly'>('weekly');
  const [scheduleTime, setScheduleTime] = useState('09:00');
  const [recipients, setRecipients] = useState<string>('');

  const handleGenerateReport = async () => {
    setIsGenerating(true);
    try {
      const config: ReportConfig = {
        title: reportTitle,
        format: reportFormat,
        includeExecutiveSummary,
        includeTechnicalDetails,
        includeCodeSnippets,
        includeScreenshots: false,
        includeRecommendations,
        includeCompliance: true,
        includeTrends,
        standards: selectedStandards,
        recipients: recipients.split(',').map(e => e.trim()).filter(Boolean),
      };

      const blob = await generateReport(config);

      // Download the report
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${reportTitle.replace(/\s+/g, '-')}.${reportFormat}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      onExport?.(reportFormat, blob);
    } catch (err) {
      console.error('Failed to generate report:', err);
      alert('Failed to generate report. Please try again.');
    } finally {
      setIsGenerating(false);
    }
  };

  const handleScheduleReport = async () => {
    try {
      const config: ReportConfig = {
        title: reportTitle,
        format: reportFormat,
        includeExecutiveSummary,
        includeTechnicalDetails,
        includeCodeSnippets,
        includeScreenshots: false,
        includeRecommendations,
        includeCompliance: true,
        includeTrends,
        standards: selectedStandards,
        recipients: recipients.split(',').map(e => e.trim()).filter(Boolean),
        schedule: {
          enabled: scheduleEnabled,
          frequency: scheduleFrequency,
          time: scheduleTime,
        },
      };

      await scheduleReport(config);
      alert('Report scheduled successfully!');
    } catch (err) {
      console.error('Failed to schedule report:', err);
      alert('Failed to schedule report. Please try again.');
    }
  };

  const handleExportData = async (format: ReportFormat) => {
    try {
      const blob = await exportData({
        format,
        filename: `accessibility-data-${Date.now()}`,
        includeAttachments: false,
      });

      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `accessibility-data.${format}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Failed to export data:', err);
      alert('Failed to export data. Please try again.');
    }
  };

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
          Compliance Reports
        </h1>
        <p style={{ color: theme.colors.text.secondary }}>
          Generate comprehensive accessibility compliance reports and schedule automated delivery
        </p>
      </div>

      <Tabs
        tabs={[
          {
            id: 'executive',
            label: 'Executive Summary',
            content: <ExecutiveSummary stats={stats} scoreData={scoreData} />,
          },
          {
            id: 'technical',
            label: 'Technical Report',
            content: <TechnicalReport issues={issues} complianceStatus={complianceStatus} />,
          },
          {
            id: 'generate',
            label: 'Generate Report',
            content: (
              <GenerateReportPanel
                reportTitle={reportTitle}
                setReportTitle={setReportTitle}
                reportFormat={reportFormat}
                setReportFormat={setReportFormat}
                selectedStandards={selectedStandards}
                setSelectedStandards={setSelectedStandards}
                includeExecutiveSummary={includeExecutiveSummary}
                setIncludeExecutiveSummary={setIncludeExecutiveSummary}
                includeTechnicalDetails={includeTechnicalDetails}
                setIncludeTechnicalDetails={setIncludeTechnicalDetails}
                includeCodeSnippets={includeCodeSnippets}
                setIncludeCodeSnippets={setIncludeCodeSnippets}
                includeRecommendations={includeRecommendations}
                setIncludeRecommendations={setIncludeRecommendations}
                includeTrends={includeTrends}
                setIncludeTrends={setIncludeTrends}
                isGenerating={isGenerating}
                onGenerate={handleGenerateReport}
              />
            ),
          },
          {
            id: 'schedule',
            label: 'Schedule Reports',
            content: (
              <ScheduleReportPanel
                scheduleEnabled={scheduleEnabled}
                setScheduleEnabled={setScheduleEnabled}
                scheduleFrequency={scheduleFrequency}
                setScheduleFrequency={setScheduleFrequency}
                scheduleTime={scheduleTime}
                setScheduleTime={setScheduleTime}
                recipients={recipients}
                setRecipients={setRecipients}
                onSchedule={handleScheduleReport}
              />
            ),
          },
          {
            id: 'export',
            label: 'Export Data',
            content: <ExportDataPanel onExport={handleExportData} />,
          },
        ]}
        variant="line"
      />
    </div>
  );
}

// Executive Summary Component
function ExecutiveSummary({ stats, scoreData }: any) {
  const { theme } = useTheme();

  return (
    <div style={{ padding: theme.spacing[6] }}>
      <h2
        style={{
          fontSize: theme.typography.fontSize.xl,
          fontWeight: theme.typography.fontWeight.semibold,
          marginBottom: theme.spacing[6],
        }}
      >
        Executive Summary
      </h2>

      {/* Key Metrics */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
          gap: theme.spacing[4],
          marginBottom: theme.spacing[6],
        }}
      >
        <MetricCard
          label="Overall Score"
          value={scoreData?.overall || 0}
          unit="/100"
          color={
            scoreData?.overall >= 90
              ? theme.colors.status.success
              : scoreData?.overall >= 70
              ? theme.colors.status.warning
              : theme.colors.status.error
          }
        />
        <MetricCard
          label="Total Issues"
          value={stats.totalIssues}
          color={theme.colors.status.info}
        />
        <MetricCard
          label="Critical Issues"
          value={stats.byLevel[IssueLevel.Critical] || 0}
          color={theme.colors.status.error}
        />
        <MetricCard
          label="Fixed Issues"
          value={stats.fixedIssues}
          color={theme.colors.status.success}
        />
      </div>

      {/* Summary Text */}
      <div
        style={{
          padding: theme.spacing[6],
          backgroundColor: theme.colors.background.secondary,
          borderRadius: theme.borderRadius.md,
          marginBottom: theme.spacing[6],
        }}
      >
        <h3
          style={{
            fontSize: theme.typography.fontSize.lg,
            fontWeight: theme.typography.fontWeight.semibold,
            marginBottom: theme.spacing[4],
          }}
        >
          Overview
        </h3>
        <p style={{ color: theme.colors.text.secondary, lineHeight: 1.6, marginBottom: theme.spacing[3] }}>
          This report provides a comprehensive analysis of the accessibility status of the application.
          The overall accessibility score is{' '}
          <strong style={{ color: theme.colors.text.primary }}>{scoreData?.overall || 0}/100</strong>,
          with a total of <strong style={{ color: theme.colors.text.primary }}>{stats.totalIssues}</strong>{' '}
          issues identified across all severity levels.
        </p>
        <p style={{ color: theme.colors.text.secondary, lineHeight: 1.6 }}>
          Of the total issues, {stats.byLevel[IssueLevel.Critical] || 0} are critical,{' '}
          {stats.byLevel[IssueLevel.Serious] || 0} are serious,{' '}
          {stats.byLevel[IssueLevel.Moderate] || 0} are moderate, and{' '}
          {stats.byLevel[IssueLevel.Minor] || 0} are minor. The team has successfully resolved{' '}
          {stats.fixedIssues} issues, with {stats.inProgressIssues} currently in progress.
        </p>
      </div>

      {/* Recommendations */}
      <div>
        <h3
          style={{
            fontSize: theme.typography.fontSize.lg,
            fontWeight: theme.typography.fontWeight.semibold,
            marginBottom: theme.spacing[4],
          }}
        >
          Key Recommendations
        </h3>
        <ul style={{ color: theme.colors.text.secondary, lineHeight: 1.8, paddingLeft: theme.spacing[6] }}>
          {stats.byLevel[IssueLevel.Critical] > 0 && (
            <li>
              <strong style={{ color: theme.colors.status.error }}>Priority:</strong> Address all{' '}
              {stats.byLevel[IssueLevel.Critical]} critical accessibility issues immediately
            </li>
          )}
          {stats.byLevel[IssueLevel.Serious] > 0 && (
            <li>
              <strong style={{ color: theme.colors.status.warning }}>High Priority:</strong> Resolve{' '}
              {stats.byLevel[IssueLevel.Serious]} serious issues within the next sprint
            </li>
          )}
          <li>Continue regular accessibility audits to maintain compliance</li>
          <li>Implement automated accessibility testing in CI/CD pipeline</li>
          <li>Provide accessibility training for development team</li>
        </ul>
      </div>
    </div>
  );
}

// Metric Card Component
function MetricCard({
  label,
  value,
  unit,
  color,
}: {
  label: string;
  value: number;
  unit?: string;
  color: string;
}) {
  const { theme } = useTheme();

  return (
    <div
      style={{
        padding: theme.spacing[4],
        backgroundColor: theme.colors.background.secondary,
        borderRadius: theme.borderRadius.md,
        borderLeft: `4px solid ${color}`,
      }}
    >
      <div
        style={{
          fontSize: theme.typography.fontSize.sm,
          color: theme.colors.text.secondary,
          marginBottom: theme.spacing[2],
        }}
      >
        {label}
      </div>
      <div
        style={{
          fontSize: theme.typography.fontSize['2xl'],
          fontWeight: theme.typography.fontWeight.bold,
          color: theme.colors.text.primary,
        }}
      >
        {value}
        {unit && <span style={{ fontSize: theme.typography.fontSize.base }}>{unit}</span>}
      </div>
    </div>
  );
}

// Technical Report Component
function TechnicalReport({ issues, complianceStatus }: any) {
  const { theme } = useTheme();

  return (
    <div style={{ padding: theme.spacing[6] }}>
      <h2
        style={{
          fontSize: theme.typography.fontSize.xl,
          fontWeight: theme.typography.fontWeight.semibold,
          marginBottom: theme.spacing[6],
        }}
      >
        Technical Report
      </h2>

      {/* Compliance Status */}
      <div style={{ marginBottom: theme.spacing[8] }}>
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
          {Array.isArray(complianceStatus) && complianceStatus.map((status: any) => (
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
                      marginBottom: theme.spacing[1],
                    }}
                  >
                    {status.standard}
                  </div>
                  <div style={{ fontSize: theme.typography.fontSize.sm, color: theme.colors.text.secondary }}>
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
          ))}
        </div>
      </div>

      {/* Issue Breakdown */}
      <div>
        <h3
          style={{
            fontSize: theme.typography.fontSize.lg,
            fontWeight: theme.typography.fontWeight.semibold,
            marginBottom: theme.spacing[4],
          }}
        >
          Issue Breakdown
        </h3>
        <div style={{ overflowX: 'auto' }}>
          <table
            style={{
              width: '100%',
              borderCollapse: 'collapse',
              backgroundColor: theme.colors.background.secondary,
            }}
          >
            <thead>
              <tr style={{ borderBottom: `2px solid ${theme.colors.border.primary}` }}>
                <th
                  style={{
                    padding: theme.spacing[3],
                    textAlign: 'left',
                    fontWeight: theme.typography.fontWeight.semibold,
                  }}
                >
                  Issue
                </th>
                <th
                  style={{
                    padding: theme.spacing[3],
                    textAlign: 'left',
                    fontWeight: theme.typography.fontWeight.semibold,
                  }}
                >
                  Severity
                </th>
                <th
                  style={{
                    padding: theme.spacing[3],
                    textAlign: 'left',
                    fontWeight: theme.typography.fontWeight.semibold,
                  }}
                >
                  Category
                </th>
                <th
                  style={{
                    padding: theme.spacing[3],
                    textAlign: 'left',
                    fontWeight: theme.typography.fontWeight.semibold,
                  }}
                >
                  Status
                </th>
              </tr>
            </thead>
            <tbody>
              {issues.slice(0, 10).map((issue: any) => (
                <tr key={issue.id} style={{ borderBottom: `1px solid ${theme.colors.border.secondary}` }}>
                  <td style={{ padding: theme.spacing[3] }}>{issue.title}</td>
                  <td style={{ padding: theme.spacing[3] }}>{issue.level}</td>
                  <td style={{ padding: theme.spacing[3] }}>{issue.category.replace(/-/g, ' ')}</td>
                  <td style={{ padding: theme.spacing[3] }}>{issue.status}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

// Generate Report Panel
function GenerateReportPanel(props: any) {
  const { theme } = useTheme();

  return (
    <div style={{ padding: theme.spacing[6] }}>
      <h3
        style={{
          fontSize: theme.typography.fontSize.lg,
          fontWeight: theme.typography.fontWeight.semibold,
          marginBottom: theme.spacing[4],
        }}
      >
        Generate Custom Report
      </h3>

      <div style={{ display: 'flex', flexDirection: 'column', gap: theme.spacing[4], maxWidth: '600px' }}>
        <div>
          <label
            htmlFor="report-title"
            style={{
              display: 'block',
              marginBottom: theme.spacing[2],
              fontWeight: theme.typography.fontWeight.medium,
            }}
          >
            Report Title
          </label>
          <Input
            id="report-title"
            type="text"
            value={props.reportTitle}
            onChange={(e) => props.setReportTitle(e.target.value)}
            placeholder="Enter report title"
          />
        </div>

        <div>
          <label
            htmlFor="report-format"
            style={{
              display: 'block',
              marginBottom: theme.spacing[2],
              fontWeight: theme.typography.fontWeight.medium,
            }}
          >
            Format
          </label>
          <Select
            id="report-format"
            value={props.reportFormat}
            onChange={(e) => props.setReportFormat(e.target.value as ReportFormat)}
          >
            <option value={ReportFormat.PDF}>PDF</option>
            <option value={ReportFormat.CSV}>CSV</option>
            <option value={ReportFormat.JSON}>JSON</option>
            <option value={ReportFormat.HTML}>HTML</option>
          </Select>
        </div>

        <div>
          <label style={{ display: 'block', marginBottom: theme.spacing[3], fontWeight: theme.typography.fontWeight.medium }}>
            Include Sections
          </label>
          <div style={{ display: 'flex', flexDirection: 'column', gap: theme.spacing[2] }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: theme.spacing[2] }}>
              <input
                type="checkbox"
                checked={props.includeExecutiveSummary}
                onChange={(e) => props.setIncludeExecutiveSummary(e.target.checked)}
              />
              Executive Summary
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: theme.spacing[2] }}>
              <input
                type="checkbox"
                checked={props.includeTechnicalDetails}
                onChange={(e) => props.setIncludeTechnicalDetails(e.target.checked)}
              />
              Technical Details
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: theme.spacing[2] }}>
              <input
                type="checkbox"
                checked={props.includeCodeSnippets}
                onChange={(e) => props.setIncludeCodeSnippets(e.target.checked)}
              />
              Code Snippets
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: theme.spacing[2] }}>
              <input
                type="checkbox"
                checked={props.includeRecommendations}
                onChange={(e) => props.setIncludeRecommendations(e.target.checked)}
              />
              Recommendations
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: theme.spacing[2] }}>
              <input
                type="checkbox"
                checked={props.includeTrends}
                onChange={(e) => props.setIncludeTrends(e.target.checked)}
              />
              Trend Charts
            </label>
          </div>
        </div>

        <Button
          variant="primary"
          size="lg"
          onClick={props.onGenerate}
          disabled={props.isGenerating}
          loading={props.isGenerating}
        >
          {props.isGenerating ? 'Generating...' : 'Generate Report'}
        </Button>
      </div>
    </div>
  );
}

// Schedule Report Panel
function ScheduleReportPanel(props: any) {
  const { theme } = useTheme();

  return (
    <div style={{ padding: theme.spacing[6] }}>
      <h3
        style={{
          fontSize: theme.typography.fontSize.lg,
          fontWeight: theme.typography.fontWeight.semibold,
          marginBottom: theme.spacing[4],
        }}
      >
        Schedule Automated Reports
      </h3>

      <div style={{ display: 'flex', flexDirection: 'column', gap: theme.spacing[4], maxWidth: '600px' }}>
        <label style={{ display: 'flex', alignItems: 'center', gap: theme.spacing[2] }}>
          <input
            type="checkbox"
            checked={props.scheduleEnabled}
            onChange={(e) => props.setScheduleEnabled(e.target.checked)}
          />
          <span style={{ fontWeight: theme.typography.fontWeight.medium }}>Enable Scheduled Reports</span>
        </label>

        {props.scheduleEnabled && (
          <>
            <div>
              <label
                htmlFor="frequency"
                style={{
                  display: 'block',
                  marginBottom: theme.spacing[2],
                  fontWeight: theme.typography.fontWeight.medium,
                }}
              >
                Frequency
              </label>
              <Select
                id="frequency"
                value={props.scheduleFrequency}
                onChange={(e) => props.setScheduleFrequency(e.target.value as any)}
              >
                <option value="daily">Daily</option>
                <option value="weekly">Weekly</option>
                <option value="monthly">Monthly</option>
              </Select>
            </div>

            <div>
              <label
                htmlFor="time"
                style={{
                  display: 'block',
                  marginBottom: theme.spacing[2],
                  fontWeight: theme.typography.fontWeight.medium,
                }}
              >
                Time
              </label>
              <Input
                id="time"
                type="time"
                value={props.scheduleTime}
                onChange={(e) => props.setScheduleTime(e.target.value)}
              />
            </div>

            <div>
              <label
                htmlFor="recipients"
                style={{
                  display: 'block',
                  marginBottom: theme.spacing[2],
                  fontWeight: theme.typography.fontWeight.medium,
                }}
              >
                Email Recipients (comma-separated)
              </label>
              <Input
                id="recipients"
                type="text"
                value={props.recipients}
                onChange={(e) => props.setRecipients(e.target.value)}
                placeholder="user@example.com, admin@example.com"
              />
            </div>

            <Button variant="primary" size="lg" onClick={props.onSchedule}>
              Save Schedule
            </Button>
          </>
        )}
      </div>
    </div>
  );
}

// Export Data Panel
function ExportDataPanel({ onExport }: { onExport: (format: ReportFormat) => void }) {
  const { theme } = useTheme();

  return (
    <div style={{ padding: theme.spacing[6] }}>
      <h3
        style={{
          fontSize: theme.typography.fontSize.lg,
          fontWeight: theme.typography.fontWeight.semibold,
          marginBottom: theme.spacing[4],
        }}
      >
        Export Raw Data
      </h3>
      <p style={{ color: theme.colors.text.secondary, marginBottom: theme.spacing[6] }}>
        Export all accessibility data in your preferred format for external analysis.
      </p>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))', gap: theme.spacing[3], maxWidth: '600px' }}>
        <Button variant="secondary" onClick={() => onExport(ReportFormat.JSON)}>
          Export as JSON
        </Button>
        <Button variant="secondary" onClick={() => onExport(ReportFormat.CSV)}>
          Export as CSV
        </Button>
        <Button variant="secondary" onClick={() => onExport(ReportFormat.XLSX)}>
          Export as Excel
        </Button>
      </div>
    </div>
  );
}
