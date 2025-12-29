/**
 * Compliance Reports Component
 * Automated compliance report generation (SOC2, GDPR, HIPAA, etc.)
 */

import React, { useState, useEffect } from 'react';
import type { ComplianceReport, ComplianceFramework } from './types';

interface ComplianceReportsProps {
  organizationId?: string;
}

export const ComplianceReports: React.FC<ComplianceReportsProps> = ({
  organizationId,
}) => {
  const [reports, setReports] = useState<ComplianceReport[]>([]);
  const [loading, setLoading] = useState(true);
  const [showGenerateModal, setShowGenerateModal] = useState(false);
  const [selectedReport, setSelectedReport] = useState<ComplianceReport | null>(null);

  useEffect(() => {
    loadReports();
  }, [organizationId]);

  const loadReports = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams(
        organizationId ? { organization_id: organizationId } : {}
      );
      const response = await fetch(`/api/compliance/reports?${params}`);
      const data = await response.json();
      setReports(data.reports || []);
    } catch (error) {
      console.error('Failed to load compliance reports:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleViewReport = (report: ComplianceReport) => {
    setSelectedReport(report);
  };

  const handleDownloadReport = async (reportId: string, format: 'pdf' | 'docx' | 'html') => {
    try {
      const response = await fetch(`/api/compliance/reports/${reportId}/download?format=${format}`);
      const blob = await response.blob();
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `compliance-report-${reportId}.${format}`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to download report:', error);
    }
  };

  if (loading) {
    return (
      <div className="compliance-reports loading">
        <div className="loading-spinner" />
        <p>Loading compliance reports...</p>
      </div>
    );
  }

  if (selectedReport) {
    return (
      <ReportViewer
        report={selectedReport}
        onBack={() => setSelectedReport(null)}
        onDownload={handleDownloadReport}
      />
    );
  }

  return (
    <div className="compliance-reports">
      {/* Header */}
      <div className="reports-header">
        <div>
          <h2>Compliance Reports</h2>
          <p className="subtitle">
            Generate and manage compliance reports for audits and certifications
          </p>
        </div>
        <button
          className="btn btn-primary"
          onClick={() => setShowGenerateModal(true)}
        >
          Generate Report
        </button>
      </div>

      {/* Report Templates */}
      <div className="report-templates">
        <h3>Available Report Templates</h3>
        <div className="templates-grid">
          <ReportTemplate
            framework="SOC2"
            title="SOC 2 Type II Report"
            description="Security, availability, processing integrity, confidentiality, and privacy"
            icon="🔒"
            onClick={() => setShowGenerateModal(true)}
          />
          <ReportTemplate
            framework="GDPR"
            title="GDPR Compliance Report"
            description="Data protection and privacy compliance documentation"
            icon="🇪🇺"
            onClick={() => setShowGenerateModal(true)}
          />
          <ReportTemplate
            framework="HIPAA"
            title="HIPAA Security Assessment"
            description="Healthcare information security and privacy safeguards"
            icon="🏥"
            onClick={() => setShowGenerateModal(true)}
          />
          <ReportTemplate
            framework="ISO27001"
            title="ISO 27001 Assessment"
            description="Information security management system certification"
            icon="📋"
            onClick={() => setShowGenerateModal(true)}
          />
        </div>
      </div>

      {/* Recent Reports */}
      <div className="recent-reports">
        <h3>Recent Reports</h3>
        {reports.length === 0 ? (
          <div className="empty-state">
            <p>No compliance reports generated yet</p>
            <button
              className="btn btn-primary"
              onClick={() => setShowGenerateModal(true)}
            >
              Generate Your First Report
            </button>
          </div>
        ) : (
          <table className="reports-table">
            <thead>
              <tr>
                <th>Framework</th>
                <th>Report Type</th>
                <th>Period</th>
                <th>Compliance</th>
                <th>Status</th>
                <th>Generated</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {reports.map((report) => (
                <tr key={report.id}>
                  <td>
                    <span className="framework-badge">{report.framework}</span>
                  </td>
                  <td>{formatReportType(report.report_type)}</td>
                  <td>
                    {new Date(report.start_date).toLocaleDateString()} -{' '}
                    {new Date(report.end_date).toLocaleDateString()}
                  </td>
                  <td>
                    <ComplianceIndicator
                      percentage={report.overall_compliance_percentage}
                    />
                  </td>
                  <td>
                    <StatusBadge status={report.status} />
                  </td>
                  <td>{new Date(report.generated_at).toLocaleDateString()}</td>
                  <td className="actions-cell">
                    <button
                      className="btn-icon"
                      onClick={() => handleViewReport(report)}
                      title="View Report"
                    >
                      👁️
                    </button>
                    <button
                      className="btn-icon"
                      onClick={() => handleDownloadReport(report.id, 'pdf')}
                      title="Download PDF"
                    >
                      📥
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Generate Report Modal */}
      {showGenerateModal && (
        <GenerateReportModal
          organizationId={organizationId}
          onClose={() => setShowGenerateModal(false)}
          onGenerated={(report) => {
            setReports((prev) => [report, ...prev]);
            setShowGenerateModal(false);
          }}
        />
      )}
    </div>
  );
};

// Report Template Component
function ReportTemplate({
  framework,
  title,
  description,
  icon,
  onClick,
}: {
  framework: string;
  title: string;
  description: string;
  icon: string;
  onClick: () => void;
}) {
  return (
    <button className="report-template" onClick={onClick}>
      <div className="template-icon">{icon}</div>
      <div className="template-content">
        <div className="template-framework">{framework}</div>
        <div className="template-title">{title}</div>
        <div className="template-description">{description}</div>
      </div>
    </button>
  );
}

// Compliance Indicator Component
function ComplianceIndicator({ percentage }: { percentage: number }) {
  const getColor = () => {
    if (percentage >= 90) return 'green';
    if (percentage >= 70) return 'yellow';
    return 'red';
  };

  return (
    <div className="compliance-indicator">
      <div className="indicator-bar">
        <div
          className={`indicator-fill fill-${getColor()}`}
          style={{ width: `${percentage}%` }}
        />
      </div>
      <span className="indicator-percentage">{percentage.toFixed(1)}%</span>
    </div>
  );
}

// Status Badge Component
function StatusBadge({ status }: { status: string }) {
  const colors: Record<string, string> = {
    draft: 'gray',
    final: 'blue',
    approved: 'green',
  };

  return (
    <span className={`badge badge-${colors[status] || 'gray'}`}>
      {status.toUpperCase()}
    </span>
  );
}

// Report Viewer Component
function ReportViewer({
  report,
  onBack,
  onDownload,
}: {
  report: ComplianceReport;
  onBack: () => void;
  onDownload: (reportId: string, format: 'pdf' | 'docx' | 'html') => void;
}) {
  return (
    <div className="report-viewer">
      {/* Header */}
      <div className="viewer-header">
        <button className="btn btn-ghost" onClick={onBack}>
          ← Back to Reports
        </button>
        <div className="header-actions">
          <button
            className="btn btn-secondary"
            onClick={() => onDownload(report.id, 'pdf')}
          >
            Download PDF
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => onDownload(report.id, 'docx')}
          >
            Download Word
          </button>
        </div>
      </div>

      {/* Report Content */}
      <div className="report-content">
        {/* Title Page */}
        <div className="report-section title-page">
          <h1>{report.framework} Compliance Report</h1>
          <h2>{formatReportType(report.report_type)}</h2>
          <div className="report-meta">
            <p>
              <strong>Period:</strong>{' '}
              {new Date(report.start_date).toLocaleDateString()} -{' '}
              {new Date(report.end_date).toLocaleDateString()}
            </p>
            <p>
              <strong>Generated:</strong>{' '}
              {new Date(report.generated_at).toLocaleDateString()}
            </p>
            <p>
              <strong>Generated By:</strong> {report.generated_by}
            </p>
            {report.approved_by && (
              <p>
                <strong>Approved By:</strong> {report.approved_by} on{' '}
                {new Date(report.approved_at!).toLocaleDateString()}
              </p>
            )}
          </div>
        </div>

        {/* Executive Summary */}
        <div className="report-section executive-summary">
          <h2>Executive Summary</h2>
          <div className="summary-stats">
            <div className="stat-card">
              <div className="stat-value">
                {report.overall_compliance_percentage.toFixed(1)}%
              </div>
              <div className="stat-label">Overall Compliance</div>
            </div>
            <div className="stat-card">
              <div className="stat-value">{report.compliant_count}</div>
              <div className="stat-label">Compliant</div>
            </div>
            <div className="stat-card">
              <div className="stat-value">{report.partial_count}</div>
              <div className="stat-label">Partial</div>
            </div>
            <div className="stat-card">
              <div className="stat-value">{report.non_compliant_count}</div>
              <div className="stat-label">Non-Compliant</div>
            </div>
          </div>

          <div className="compliance-chart">
            <div className="chart-bar">
              <div
                className="bar-segment compliant"
                style={{
                  width: `${(report.compliant_count / report.total_requirements) * 100}%`,
                }}
              />
              <div
                className="bar-segment partial"
                style={{
                  width: `${(report.partial_count / report.total_requirements) * 100}%`,
                }}
              />
              <div
                className="bar-segment non-compliant"
                style={{
                  width: `${(report.non_compliant_count / report.total_requirements) * 100}%`,
                }}
              />
            </div>
          </div>
        </div>

        {/* Findings */}
        {report.findings.length > 0 && (
          <div className="report-section findings">
            <h2>Key Findings</h2>
            {report.findings.map((finding, index) => (
              <div key={index} className={`finding finding-${finding.severity}`}>
                <div className="finding-header">
                  <span className={`severity-badge severity-${finding.severity}`}>
                    {finding.severity.toUpperCase()}
                  </span>
                  <h3>{finding.title}</h3>
                </div>
                <p className="finding-description">{finding.description}</p>
                <div className="finding-requirement">
                  <strong>Requirement:</strong> {finding.requirement_id}
                </div>
                {finding.evidence.length > 0 && (
                  <div className="finding-evidence">
                    <strong>Evidence:</strong>
                    <ul>
                      {finding.evidence.map((evidence, idx) => (
                        <li key={idx}>{evidence}</li>
                      ))}
                    </ul>
                  </div>
                )}
                <div className="finding-recommendation">
                  <strong>Recommendation:</strong> {finding.recommendation}
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Requirements */}
        <div className="report-section requirements">
          <h2>Compliance Requirements</h2>
          <table className="requirements-table">
            <thead>
              <tr>
                <th>ID</th>
                <th>Requirement</th>
                <th>Category</th>
                <th>Status</th>
                <th>Compliance</th>
              </tr>
            </thead>
            <tbody>
              {report.requirements.map((req) => (
                <tr key={req.id}>
                  <td>{req.requirement_id}</td>
                  <td>{req.title}</td>
                  <td>{req.category}</td>
                  <td>
                    <StatusBadge status={req.status} />
                  </td>
                  <td>{req.compliance_percentage.toFixed(0)}%</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

// Generate Report Modal
function GenerateReportModal({
  organizationId,
  onClose,
  onGenerated,
}: {
  organizationId?: string;
  onClose: () => void;
  onGenerated: (report: ComplianceReport) => void;
}) {
  const [framework, setFramework] = useState<ComplianceFramework>('SOC2');
  const [reportType, setReportType] = useState<'audit' | 'assessment' | 'certification' | 'gap_analysis'>('audit');
  const [startDate, setStartDate] = useState(
    new Date(Date.now() - 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
  );
  const [endDate, setEndDate] = useState(new Date().toISOString().split('T')[0]);
  const [generating, setGenerating] = useState(false);

  const handleGenerate = async () => {
    setGenerating(true);
    try {
      const response = await fetch('/api/compliance/reports/generate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          framework,
          report_type: reportType,
          start_date: startDate,
          end_date: endDate,
          organization_id: organizationId,
        }),
      });

      const report = await response.json();
      onGenerated(report);
    } catch (error) {
      console.error('Failed to generate report:', error);
    } finally {
      setGenerating(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal generate-report-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Generate Compliance Report</h2>
          <button className="modal-close" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="modal-content">
          <div className="form-section">
            <label>Framework</label>
            <select
              value={framework}
              onChange={(e) => setFramework(e.target.value as ComplianceFramework)}
            >
              {(['SOC2', 'GDPR', 'HIPAA', 'ISO27001', 'PCI_DSS', 'CCPA', 'NIST', 'FedRAMP'] as ComplianceFramework[]).map(
                (fw) => (
                  <option key={fw} value={fw}>
                    {fw}
                  </option>
                )
              )}
            </select>
          </div>

          <div className="form-section">
            <label>Report Type</label>
            <select
              value={reportType}
              onChange={(e) => setReportType(e.target.value as any)}
            >
              <option value="audit">Audit Report</option>
              <option value="assessment">Assessment Report</option>
              <option value="certification">Certification Report</option>
              <option value="gap_analysis">Gap Analysis</option>
            </select>
          </div>

          <div className="form-row">
            <div className="form-section">
              <label>Start Date</label>
              <input
                type="date"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
              />
            </div>
            <div className="form-section">
              <label>End Date</label>
              <input
                type="date"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
              />
            </div>
          </div>
        </div>

        <div className="modal-footer">
          <button className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button
            className="btn btn-primary"
            onClick={handleGenerate}
            disabled={generating}
          >
            {generating ? 'Generating...' : 'Generate Report'}
          </button>
        </div>
      </div>
    </div>
  );
}

// Utility Functions
function formatReportType(type: string): string {
  return type
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}
