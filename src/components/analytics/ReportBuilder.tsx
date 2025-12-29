/**
 * Report Builder
 *
 * Custom report builder with templates and scheduling.
 */

import React, { useState, useMemo } from 'react';
import { useReportGenerator, useUsageStats, usePerformanceProfiles } from './useAnalytics';
import type { ReportType, ReportFormat, TimeRange } from './types';

interface ReportConfig {
  type: ReportType;
  format: ReportFormat;
  title: string;
  description: string;
  timeRange: TimeRange;
  includeCharts: boolean;
  includeTables: boolean;
  sections: string[];
}

export function ReportBuilder() {
  const { generateReport, downloadReport, generating, error } = useReportGenerator();
  const { stats } = useUsageStats();
  const { profiles } = usePerformanceProfiles();

  const [config, setConfig] = useState<ReportConfig>({
    type: ReportType.Usage,
    format: ReportFormat.Html,
    title: 'Analytics Report',
    description: '',
    timeRange: {
      start: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000), // Last 7 days
      end: new Date(),
    },
    includeCharts: true,
    includeTables: true,
    sections: ['summary', 'metrics', 'usage', 'performance'],
  });

  const [generatedReport, setGeneratedReport] = useState<any>(null);
  const [showPreview, setShowPreview] = useState(false);

  // Available report templates
  const templates = [
    {
      id: 'executive',
      name: 'Executive Summary',
      type: ReportType.ExecutiveSummary,
      sections: ['summary', 'key_metrics', 'recommendations'],
    },
    {
      id: 'detailed',
      name: 'Detailed Analytics',
      type: ReportType.DetailedAnalytics,
      sections: ['summary', 'metrics', 'usage', 'performance', 'errors'],
    },
    {
      id: 'performance',
      name: 'Performance Report',
      type: ReportType.Performance,
      sections: ['performance', 'slowest_operations', 'errors'],
    },
    {
      id: 'usage',
      name: 'Usage Report',
      type: ReportType.Usage,
      sections: ['usage', 'features', 'commands', 'sessions'],
    },
  ];

  // Handle template selection
  const applyTemplate = (templateId: string) => {
    const template = templates.find((t) => t.id === templateId);
    if (template) {
      setConfig((prev) => ({
        ...prev,
        type: template.type,
        title: template.name,
        sections: template.sections,
      }));
    }
  };

  // Handle report generation
  const handleGenerate = async () => {
    try {
      const report = await generateReport(config.type, config.format, {
        title: config.title,
        description: config.description,
        timeRange: config.timeRange,
        includeCharts: config.includeCharts,
        includeTables: config.includeTables,
        sections: config.sections,
      });

      if (report) {
        setGeneratedReport(report);
        setShowPreview(true);
      }
    } catch (err) {
      console.error('Failed to generate report:', err);
    }
  };

  // Handle report download
  const handleDownload = async () => {
    if (generatedReport) {
      await downloadReport(generatedReport, config.format);
    }
  };

  return (
    <div className="report-builder">
      {/* Header */}
      <div className="builder-header">
        <h2>Report Builder</h2>
        <p>Create custom analytics reports with various formats and configurations</p>
      </div>

      {/* Template Selection */}
      <div className="template-selection">
        <h3>Report Templates</h3>
        <div className="template-grid">
          {templates.map((template) => (
            <TemplateCard
              key={template.id}
              name={template.name}
              onClick={() => applyTemplate(template.id)}
              selected={config.type === template.type}
            />
          ))}
        </div>
      </div>

      {/* Configuration Form */}
      <div className="report-config-form">
        <h3>Report Configuration</h3>

        <div className="form-grid">
          {/* Report Type */}
          <div className="form-group">
            <label>Report Type</label>
            <select
              value={config.type}
              onChange={(e) =>
                setConfig({ ...config, type: e.target.value as ReportType })
              }
            >
              <option value={ReportType.Usage}>Usage Report</option>
              <option value={ReportType.Performance}>Performance Report</option>
              <option value={ReportType.Errors}>Error Analysis</option>
              <option value={ReportType.ExecutiveSummary}>Executive Summary</option>
              <option value={ReportType.DetailedAnalytics}>Detailed Analytics</option>
              <option value={ReportType.Custom}>Custom Report</option>
            </select>
          </div>

          {/* Report Format */}
          <div className="form-group">
            <label>Output Format</label>
            <select
              value={config.format}
              onChange={(e) =>
                setConfig({ ...config, format: e.target.value as ReportFormat })
              }
            >
              <option value={ReportFormat.Html}>HTML</option>
              <option value={ReportFormat.Pdf}>PDF</option>
              <option value={ReportFormat.Markdown}>Markdown</option>
              <option value={ReportFormat.Json}>JSON</option>
              <option value={ReportFormat.Csv}>CSV</option>
              <option value={ReportFormat.Text}>Plain Text</option>
            </select>
          </div>

          {/* Title */}
          <div className="form-group full-width">
            <label>Report Title</label>
            <input
              type="text"
              value={config.title}
              onChange={(e) => setConfig({ ...config, title: e.target.value })}
              placeholder="Enter report title"
            />
          </div>

          {/* Description */}
          <div className="form-group full-width">
            <label>Description (Optional)</label>
            <textarea
              value={config.description}
              onChange={(e) => setConfig({ ...config, description: e.target.value })}
              placeholder="Enter report description"
              rows={3}
            />
          </div>

          {/* Time Range */}
          <div className="form-group">
            <label>Start Date</label>
            <input
              type="datetime-local"
              value={config.timeRange.start.toISOString().slice(0, 16)}
              onChange={(e) =>
                setConfig({
                  ...config,
                  timeRange: {
                    ...config.timeRange,
                    start: new Date(e.target.value),
                  },
                })
              }
            />
          </div>

          <div className="form-group">
            <label>End Date</label>
            <input
              type="datetime-local"
              value={config.timeRange.end.toISOString().slice(0, 16)}
              onChange={(e) =>
                setConfig({
                  ...config,
                  timeRange: {
                    ...config.timeRange,
                    end: new Date(e.target.value),
                  },
                })
              }
            />
          </div>

          {/* Options */}
          <div className="form-group full-width">
            <label>
              <input
                type="checkbox"
                checked={config.includeCharts}
                onChange={(e) =>
                  setConfig({ ...config, includeCharts: e.target.checked })
                }
              />
              Include Charts
            </label>
          </div>

          <div className="form-group full-width">
            <label>
              <input
                type="checkbox"
                checked={config.includeTables}
                onChange={(e) =>
                  setConfig({ ...config, includeTables: e.target.checked })
                }
              />
              Include Tables
            </label>
          </div>
        </div>

        {/* Sections Selection */}
        <div className="sections-selection">
          <h4>Report Sections</h4>
          <div className="sections-grid">
            {[
              { id: 'summary', label: 'Summary' },
              { id: 'metrics', label: 'Metrics' },
              { id: 'usage', label: 'Usage Statistics' },
              { id: 'performance', label: 'Performance' },
              { id: 'errors', label: 'Error Analysis' },
              { id: 'features', label: 'Feature Usage' },
              { id: 'commands', label: 'Command Execution' },
              { id: 'sessions', label: 'Session Data' },
              { id: 'recommendations', label: 'Recommendations' },
            ].map((section) => (
              <label key={section.id} className="section-checkbox">
                <input
                  type="checkbox"
                  checked={config.sections.includes(section.id)}
                  onChange={(e) => {
                    if (e.target.checked) {
                      setConfig({
                        ...config,
                        sections: [...config.sections, section.id],
                      });
                    } else {
                      setConfig({
                        ...config,
                        sections: config.sections.filter((s) => s !== section.id),
                      });
                    }
                  }}
                />
                {section.label}
              </label>
            ))}
          </div>
        </div>
      </div>

      {/* Preview Section */}
      <div className="report-preview-section">
        <h3>Report Preview</h3>
        <div className="preview-stats">
          <div className="preview-stat">
            <span>Data Points:</span>
            <strong>{stats?.total_events || 0}</strong>
          </div>
          <div className="preview-stat">
            <span>Time Range:</span>
            <strong>
              {Math.ceil(
                (config.timeRange.end.getTime() - config.timeRange.start.getTime()) /
                  (1000 * 60 * 60 * 24)
              )}{' '}
              days
            </strong>
          </div>
          <div className="preview-stat">
            <span>Sections:</span>
            <strong>{config.sections.length}</strong>
          </div>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="builder-actions">
        <button
          onClick={handleGenerate}
          disabled={generating}
          className="btn btn-primary"
        >
          {generating ? 'Generating...' : 'Generate Report'}
        </button>

        {generatedReport && (
          <>
            <button onClick={handleDownload} className="btn btn-success">
              Download Report
            </button>
            <button
              onClick={() => setShowPreview(!showPreview)}
              className="btn btn-secondary"
            >
              {showPreview ? 'Hide Preview' : 'Show Preview'}
            </button>
          </>
        )}
      </div>

      {/* Error Display */}
      {error && (
        <div className="error-message">
          <p>Error generating report: {error.message}</p>
        </div>
      )}

      {/* Report Preview Modal */}
      {showPreview && generatedReport && (
        <ReportPreview
          report={generatedReport}
          onClose={() => setShowPreview(false)}
        />
      )}
    </div>
  );
}

// Template Card Component
function TemplateCard({
  name,
  onClick,
  selected,
}: {
  name: string;
  onClick: () => void;
  selected: boolean;
}) {
  return (
    <div
      className={`template-card ${selected ? 'selected' : ''}`}
      onClick={onClick}
    >
      <div className="template-icon">📊</div>
      <div className="template-name">{name}</div>
    </div>
  );
}

// Report Preview Modal
function ReportPreview({ report, onClose }: { report: any; onClose: () => void }) {
  return (
    <div className="report-preview-modal">
      <div className="modal-overlay" onClick={onClose} />
      <div className="modal-content large">
        <div className="modal-header">
          <h3>Report Preview: {report.title}</h3>
          <button onClick={onClose} className="close-btn">
            ✕
          </button>
        </div>

        <div className="modal-body">
          <div className="report-metadata">
            <p>
              <strong>Type:</strong> {report.report_type}
            </p>
            <p>
              <strong>Generated:</strong>{' '}
              {new Date(report.generated_at).toLocaleString()}
            </p>
            {report.description && (
              <p>
                <strong>Description:</strong> {report.description}
              </p>
            )}
          </div>

          <div className="report-sections">
            {report.sections.map((section: any, index: number) => (
              <div key={index} className="report-section">
                <h4>{section.title}</h4>
                <pre className="section-content">{section.content}</pre>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * Scheduled Reports Component
 */
export function ScheduledReports() {
  const [schedules, setSchedules] = useState<any[]>([]);

  return (
    <div className="scheduled-reports">
      <h3>Scheduled Reports</h3>
      <p>Configure automatic report generation and delivery</p>

      {schedules.length === 0 ? (
        <div className="empty-state">
          <p>No scheduled reports configured</p>
          <button className="btn btn-primary">Create Schedule</button>
        </div>
      ) : (
        <div className="schedules-list">
          {schedules.map((schedule, index) => (
            <div key={index} className="schedule-item">
              <div>{schedule.name}</div>
              <div>{schedule.frequency}</div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
