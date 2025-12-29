import React, { useState, useEffect } from 'react';
export const ComplianceReports = ({ organizationId, }) => {
    const [reports, setReports] = useState([]);
    const [loading, setLoading] = useState(true);
    const [showGenerateModal, setShowGenerateModal] = useState(false);
    const [selectedReport, setSelectedReport] = useState(null);
    useEffect(() => {
        loadReports();
    }, [organizationId]);
    const loadReports = async () => {
        setLoading(true);
        try {
            const params = new URLSearchParams(organizationId ? { organization_id: organizationId } : {});
            const response = await fetch(`/api/compliance/reports?${params}`);
            const data = await response.json();
            setReports(data.reports || []);
        }
        catch (error) {
            console.error('Failed to load compliance reports:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const handleViewReport = (report) => {
        setSelectedReport(report);
    };
    const handleDownloadReport = async (reportId, format) => {
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
        }
        catch (error) {
            console.error('Failed to download report:', error);
        }
    };
    if (loading) {
        return (React.createElement("div", { className: "compliance-reports loading" },
            React.createElement("div", { className: "loading-spinner" }),
            React.createElement("p", null, "Loading compliance reports...")));
    }
    if (selectedReport) {
        return (React.createElement(ReportViewer, { report: selectedReport, onBack: () => setSelectedReport(null), onDownload: handleDownloadReport }));
    }
    return (React.createElement("div", { className: "compliance-reports" },
        React.createElement("div", { className: "reports-header" },
            React.createElement("div", null,
                React.createElement("h2", null, "Compliance Reports"),
                React.createElement("p", { className: "subtitle" }, "Generate and manage compliance reports for audits and certifications")),
            React.createElement("button", { className: "btn btn-primary", onClick: () => setShowGenerateModal(true) }, "Generate Report")),
        React.createElement("div", { className: "report-templates" },
            React.createElement("h3", null, "Available Report Templates"),
            React.createElement("div", { className: "templates-grid" },
                React.createElement(ReportTemplate, { framework: "SOC2", title: "SOC 2 Type II Report", description: "Security, availability, processing integrity, confidentiality, and privacy", icon: "\uD83D\uDD12", onClick: () => setShowGenerateModal(true) }),
                React.createElement(ReportTemplate, { framework: "GDPR", title: "GDPR Compliance Report", description: "Data protection and privacy compliance documentation", icon: "\uD83C\uDDEA\uD83C\uDDFA", onClick: () => setShowGenerateModal(true) }),
                React.createElement(ReportTemplate, { framework: "HIPAA", title: "HIPAA Security Assessment", description: "Healthcare information security and privacy safeguards", icon: "\uD83C\uDFE5", onClick: () => setShowGenerateModal(true) }),
                React.createElement(ReportTemplate, { framework: "ISO27001", title: "ISO 27001 Assessment", description: "Information security management system certification", icon: "\uD83D\uDCCB", onClick: () => setShowGenerateModal(true) }))),
        React.createElement("div", { className: "recent-reports" },
            React.createElement("h3", null, "Recent Reports"),
            reports.length === 0 ? (React.createElement("div", { className: "empty-state" },
                React.createElement("p", null, "No compliance reports generated yet"),
                React.createElement("button", { className: "btn btn-primary", onClick: () => setShowGenerateModal(true) }, "Generate Your First Report"))) : (React.createElement("table", { className: "reports-table" },
                React.createElement("thead", null,
                    React.createElement("tr", null,
                        React.createElement("th", null, "Framework"),
                        React.createElement("th", null, "Report Type"),
                        React.createElement("th", null, "Period"),
                        React.createElement("th", null, "Compliance"),
                        React.createElement("th", null, "Status"),
                        React.createElement("th", null, "Generated"),
                        React.createElement("th", null, "Actions"))),
                React.createElement("tbody", null, reports.map((report) => (React.createElement("tr", { key: report.id },
                    React.createElement("td", null,
                        React.createElement("span", { className: "framework-badge" }, report.framework)),
                    React.createElement("td", null, formatReportType(report.report_type)),
                    React.createElement("td", null,
                        new Date(report.start_date).toLocaleDateString(),
                        " -",
                        ' ',
                        new Date(report.end_date).toLocaleDateString()),
                    React.createElement("td", null,
                        React.createElement(ComplianceIndicator, { percentage: report.overall_compliance_percentage })),
                    React.createElement("td", null,
                        React.createElement(StatusBadge, { status: report.status })),
                    React.createElement("td", null, new Date(report.generated_at).toLocaleDateString()),
                    React.createElement("td", { className: "actions-cell" },
                        React.createElement("button", { className: "btn-icon", onClick: () => handleViewReport(report), title: "View Report" }, "\uD83D\uDC41\uFE0F"),
                        React.createElement("button", { className: "btn-icon", onClick: () => handleDownloadReport(report.id, 'pdf'), title: "Download PDF" }, "\uD83D\uDCE5"))))))))),
        showGenerateModal && (React.createElement(GenerateReportModal, { organizationId: organizationId, onClose: () => setShowGenerateModal(false), onGenerated: (report) => {
                setReports((prev) => [report, ...prev]);
                setShowGenerateModal(false);
            } }))));
};
function ReportTemplate({ framework, title, description, icon, onClick, }) {
    return (React.createElement("button", { className: "report-template", onClick: onClick },
        React.createElement("div", { className: "template-icon" }, icon),
        React.createElement("div", { className: "template-content" },
            React.createElement("div", { className: "template-framework" }, framework),
            React.createElement("div", { className: "template-title" }, title),
            React.createElement("div", { className: "template-description" }, description))));
}
function ComplianceIndicator({ percentage }) {
    const getColor = () => {
        if (percentage >= 90)
            return 'green';
        if (percentage >= 70)
            return 'yellow';
        return 'red';
    };
    return (React.createElement("div", { className: "compliance-indicator" },
        React.createElement("div", { className: "indicator-bar" },
            React.createElement("div", { className: `indicator-fill fill-${getColor()}`, style: { width: `${percentage}%` } })),
        React.createElement("span", { className: "indicator-percentage" },
            percentage.toFixed(1),
            "%")));
}
function StatusBadge({ status }) {
    const colors = {
        draft: 'gray',
        final: 'blue',
        approved: 'green',
    };
    return (React.createElement("span", { className: `badge badge-${colors[status] || 'gray'}` }, status.toUpperCase()));
}
function ReportViewer({ report, onBack, onDownload, }) {
    return (React.createElement("div", { className: "report-viewer" },
        React.createElement("div", { className: "viewer-header" },
            React.createElement("button", { className: "btn btn-ghost", onClick: onBack }, "\u2190 Back to Reports"),
            React.createElement("div", { className: "header-actions" },
                React.createElement("button", { className: "btn btn-secondary", onClick: () => onDownload(report.id, 'pdf') }, "Download PDF"),
                React.createElement("button", { className: "btn btn-secondary", onClick: () => onDownload(report.id, 'docx') }, "Download Word"))),
        React.createElement("div", { className: "report-content" },
            React.createElement("div", { className: "report-section title-page" },
                React.createElement("h1", null,
                    report.framework,
                    " Compliance Report"),
                React.createElement("h2", null, formatReportType(report.report_type)),
                React.createElement("div", { className: "report-meta" },
                    React.createElement("p", null,
                        React.createElement("strong", null, "Period:"),
                        ' ',
                        new Date(report.start_date).toLocaleDateString(),
                        " -",
                        ' ',
                        new Date(report.end_date).toLocaleDateString()),
                    React.createElement("p", null,
                        React.createElement("strong", null, "Generated:"),
                        ' ',
                        new Date(report.generated_at).toLocaleDateString()),
                    React.createElement("p", null,
                        React.createElement("strong", null, "Generated By:"),
                        " ",
                        report.generated_by),
                    report.approved_by && (React.createElement("p", null,
                        React.createElement("strong", null, "Approved By:"),
                        " ",
                        report.approved_by,
                        " on",
                        ' ',
                        new Date(report.approved_at).toLocaleDateString())))),
            React.createElement("div", { className: "report-section executive-summary" },
                React.createElement("h2", null, "Executive Summary"),
                React.createElement("div", { className: "summary-stats" },
                    React.createElement("div", { className: "stat-card" },
                        React.createElement("div", { className: "stat-value" },
                            report.overall_compliance_percentage.toFixed(1),
                            "%"),
                        React.createElement("div", { className: "stat-label" }, "Overall Compliance")),
                    React.createElement("div", { className: "stat-card" },
                        React.createElement("div", { className: "stat-value" }, report.compliant_count),
                        React.createElement("div", { className: "stat-label" }, "Compliant")),
                    React.createElement("div", { className: "stat-card" },
                        React.createElement("div", { className: "stat-value" }, report.partial_count),
                        React.createElement("div", { className: "stat-label" }, "Partial")),
                    React.createElement("div", { className: "stat-card" },
                        React.createElement("div", { className: "stat-value" }, report.non_compliant_count),
                        React.createElement("div", { className: "stat-label" }, "Non-Compliant"))),
                React.createElement("div", { className: "compliance-chart" },
                    React.createElement("div", { className: "chart-bar" },
                        React.createElement("div", { className: "bar-segment compliant", style: {
                                width: `${(report.compliant_count / report.total_requirements) * 100}%`,
                            } }),
                        React.createElement("div", { className: "bar-segment partial", style: {
                                width: `${(report.partial_count / report.total_requirements) * 100}%`,
                            } }),
                        React.createElement("div", { className: "bar-segment non-compliant", style: {
                                width: `${(report.non_compliant_count / report.total_requirements) * 100}%`,
                            } })))),
            report.findings.length > 0 && (React.createElement("div", { className: "report-section findings" },
                React.createElement("h2", null, "Key Findings"),
                report.findings.map((finding, index) => (React.createElement("div", { key: index, className: `finding finding-${finding.severity}` },
                    React.createElement("div", { className: "finding-header" },
                        React.createElement("span", { className: `severity-badge severity-${finding.severity}` }, finding.severity.toUpperCase()),
                        React.createElement("h3", null, finding.title)),
                    React.createElement("p", { className: "finding-description" }, finding.description),
                    React.createElement("div", { className: "finding-requirement" },
                        React.createElement("strong", null, "Requirement:"),
                        " ",
                        finding.requirement_id),
                    finding.evidence.length > 0 && (React.createElement("div", { className: "finding-evidence" },
                        React.createElement("strong", null, "Evidence:"),
                        React.createElement("ul", null, finding.evidence.map((evidence, idx) => (React.createElement("li", { key: idx }, evidence)))))),
                    React.createElement("div", { className: "finding-recommendation" },
                        React.createElement("strong", null, "Recommendation:"),
                        " ",
                        finding.recommendation)))))),
            React.createElement("div", { className: "report-section requirements" },
                React.createElement("h2", null, "Compliance Requirements"),
                React.createElement("table", { className: "requirements-table" },
                    React.createElement("thead", null,
                        React.createElement("tr", null,
                            React.createElement("th", null, "ID"),
                            React.createElement("th", null, "Requirement"),
                            React.createElement("th", null, "Category"),
                            React.createElement("th", null, "Status"),
                            React.createElement("th", null, "Compliance"))),
                    React.createElement("tbody", null, report.requirements.map((req) => (React.createElement("tr", { key: req.id },
                        React.createElement("td", null, req.requirement_id),
                        React.createElement("td", null, req.title),
                        React.createElement("td", null, req.category),
                        React.createElement("td", null,
                            React.createElement(StatusBadge, { status: req.status })),
                        React.createElement("td", null,
                            req.compliance_percentage.toFixed(0),
                            "%"))))))))));
}
function GenerateReportModal({ organizationId, onClose, onGenerated, }) {
    const [framework, setFramework] = useState('SOC2');
    const [reportType, setReportType] = useState('audit');
    const [startDate, setStartDate] = useState(new Date(Date.now() - 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]);
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
        }
        catch (error) {
            console.error('Failed to generate report:', error);
        }
        finally {
            setGenerating(false);
        }
    };
    return (React.createElement("div", { className: "modal-overlay", onClick: onClose },
        React.createElement("div", { className: "modal generate-report-modal", onClick: (e) => e.stopPropagation() },
            React.createElement("div", { className: "modal-header" },
                React.createElement("h2", null, "Generate Compliance Report"),
                React.createElement("button", { className: "modal-close", onClick: onClose }, "\u00D7")),
            React.createElement("div", { className: "modal-content" },
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Framework"),
                    React.createElement("select", { value: framework, onChange: (e) => setFramework(e.target.value) }, ['SOC2', 'GDPR', 'HIPAA', 'ISO27001', 'PCI_DSS', 'CCPA', 'NIST', 'FedRAMP'].map((fw) => (React.createElement("option", { key: fw, value: fw }, fw))))),
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Report Type"),
                    React.createElement("select", { value: reportType, onChange: (e) => setReportType(e.target.value) },
                        React.createElement("option", { value: "audit" }, "Audit Report"),
                        React.createElement("option", { value: "assessment" }, "Assessment Report"),
                        React.createElement("option", { value: "certification" }, "Certification Report"),
                        React.createElement("option", { value: "gap_analysis" }, "Gap Analysis"))),
                React.createElement("div", { className: "form-row" },
                    React.createElement("div", { className: "form-section" },
                        React.createElement("label", null, "Start Date"),
                        React.createElement("input", { type: "date", value: startDate, onChange: (e) => setStartDate(e.target.value) })),
                    React.createElement("div", { className: "form-section" },
                        React.createElement("label", null, "End Date"),
                        React.createElement("input", { type: "date", value: endDate, onChange: (e) => setEndDate(e.target.value) })))),
            React.createElement("div", { className: "modal-footer" },
                React.createElement("button", { className: "btn btn-secondary", onClick: onClose }, "Cancel"),
                React.createElement("button", { className: "btn btn-primary", onClick: handleGenerate, disabled: generating }, generating ? 'Generating...' : 'Generate Report')))));
}
function formatReportType(type) {
    return type
        .split('_')
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ');
}
//# sourceMappingURL=ComplianceReports.js.map