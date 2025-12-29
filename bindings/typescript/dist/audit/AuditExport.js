import React, { useState } from 'react';
import { AuditFilters } from './AuditFilters';
export const AuditExport = ({ organizationId }) => {
    const [filters, setFilters] = useState({});
    const [exportFormat, setExportFormat] = useState('csv');
    const [includeMetadata, setIncludeMetadata] = useState(true);
    const [includeHashChain, setIncludeHashChain] = useState(true);
    const [encrypt, setEncrypt] = useState(false);
    const [password, setPassword] = useState('');
    const [digitalSignature, setDigitalSignature] = useState(true);
    const [exporting, setExporting] = useState(false);
    const [exportProgress, setExportProgress] = useState(0);
    const [exportHistory, setExportHistory] = useState([]);
    const handleExport = async () => {
        setExporting(true);
        setExportProgress(0);
        try {
            const options = {
                format: exportFormat,
                filters,
                include_metadata: includeMetadata,
                include_hash_chain: includeHashChain,
                encrypt,
                password: encrypt ? password : undefined,
                digital_signature: digitalSignature,
            };
            const response = await fetch('/api/audit/export', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    ...options,
                    organization_id: organizationId,
                }),
            });
            if (!response.ok) {
                throw new Error('Export failed');
            }
            const progressInterval = setInterval(() => {
                setExportProgress((prev) => Math.min(prev + 10, 90));
            }, 200);
            const blob = await response.blob();
            clearInterval(progressInterval);
            setExportProgress(100);
            const url = URL.createObjectURL(blob);
            const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
            const filename = `audit-export-${timestamp}.${exportFormat}${encrypt ? '.encrypted' : ''}`;
            const link = document.createElement('a');
            link.href = url;
            link.download = filename;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            URL.revokeObjectURL(url);
            const historyItem = {
                id: crypto.randomUUID(),
                timestamp: new Date().toISOString(),
                format: exportFormat,
                filename,
                event_count: 0,
                encrypted: encrypt,
                signed: digitalSignature,
            };
            setExportHistory((prev) => [historyItem, ...prev].slice(0, 10));
            setTimeout(() => {
                setExportProgress(0);
                setExporting(false);
            }, 1000);
        }
        catch (error) {
            console.error('Export failed:', error);
            setExporting(false);
            setExportProgress(0);
            alert('Export failed. Please try again.');
        }
    };
    const isValid = () => {
        if (encrypt && !password)
            return false;
        return true;
    };
    return (React.createElement("div", { className: "audit-export" },
        React.createElement("div", { className: "export-header" },
            React.createElement("div", null,
                React.createElement("h2", null, "Export Audit Logs"),
                React.createElement("p", { className: "subtitle" }, "Export audit logs for compliance, archival, or analysis purposes"))),
        React.createElement("div", { className: "export-content" },
            React.createElement("div", { className: "export-config" },
                React.createElement("section", { className: "config-section" },
                    React.createElement("h3", null, "Export Format"),
                    React.createElement("div", { className: "format-options" },
                        React.createElement(FormatOption, { format: "csv", label: "CSV", description: "Comma-separated values for spreadsheet analysis", icon: "\uD83D\uDCCA", selected: exportFormat === 'csv', onClick: () => setExportFormat('csv') }),
                        React.createElement(FormatOption, { format: "json", label: "JSON", description: "Structured JSON for programmatic processing", icon: "{ }", selected: exportFormat === 'json', onClick: () => setExportFormat('json') }),
                        React.createElement(FormatOption, { format: "pdf", label: "PDF", description: "Formatted PDF report for documentation", icon: "\uD83D\uDCC4", selected: exportFormat === 'pdf', onClick: () => setExportFormat('pdf') }),
                        React.createElement(FormatOption, { format: "xlsx", label: "Excel", description: "Excel workbook with multiple sheets", icon: "\uD83D\uDCC8", selected: exportFormat === 'xlsx', onClick: () => setExportFormat('xlsx') }))),
                React.createElement("section", { className: "config-section" },
                    React.createElement("h3", null, "Export Options"),
                    React.createElement("div", { className: "options-list" },
                        React.createElement("label", { className: "option-item" },
                            React.createElement("input", { type: "checkbox", checked: includeMetadata, onChange: (e) => setIncludeMetadata(e.target.checked) }),
                            React.createElement("div", { className: "option-content" },
                                React.createElement("strong", null, "Include Metadata"),
                                React.createElement("span", { className: "option-description" }, "Include custom metadata and additional context"))),
                        React.createElement("label", { className: "option-item" },
                            React.createElement("input", { type: "checkbox", checked: includeHashChain, onChange: (e) => setIncludeHashChain(e.target.checked) }),
                            React.createElement("div", { className: "option-content" },
                                React.createElement("strong", null, "Include Hash Chain"),
                                React.createElement("span", { className: "option-description" }, "Include cryptographic hashes for integrity verification"))),
                        React.createElement("label", { className: "option-item" },
                            React.createElement("input", { type: "checkbox", checked: digitalSignature, onChange: (e) => setDigitalSignature(e.target.checked) }),
                            React.createElement("div", { className: "option-content" },
                                React.createElement("strong", null, "Digital Signature"),
                                React.createElement("span", { className: "option-description" }, "Sign export with organization's private key"))),
                        React.createElement("label", { className: "option-item" },
                            React.createElement("input", { type: "checkbox", checked: encrypt, onChange: (e) => {
                                    setEncrypt(e.target.checked);
                                    if (!e.target.checked)
                                        setPassword('');
                                } }),
                            React.createElement("div", { className: "option-content" },
                                React.createElement("strong", null, "Encrypt Export"),
                                React.createElement("span", { className: "option-description" }, "Password-protect the exported file with AES-256 encryption"))),
                        encrypt && (React.createElement("div", { className: "password-input" },
                            React.createElement("input", { type: "password", placeholder: "Enter encryption password", value: password, onChange: (e) => setPassword(e.target.value), className: "input" }),
                            React.createElement("small", { className: "input-hint" }, "Minimum 12 characters. Store this password securely."))))),
                React.createElement("section", { className: "config-section" },
                    React.createElement("h3", null, "Filters"),
                    React.createElement(AuditFilters, { filters: filters, onChange: setFilters, onReset: () => setFilters({}) })),
                React.createElement("div", { className: "export-actions" },
                    React.createElement("button", { className: "btn btn-primary btn-lg", onClick: handleExport, disabled: !isValid() || exporting }, exporting ? 'Exporting...' : 'Export Audit Logs')),
                exporting && (React.createElement("div", { className: "export-progress" },
                    React.createElement("div", { className: "progress-bar" },
                        React.createElement("div", { className: "progress-fill", style: { width: `${exportProgress}%` } })),
                    React.createElement("span", { className: "progress-text" },
                        exportProgress,
                        "%")))),
            React.createElement("div", { className: "export-history" },
                React.createElement("h3", null, "Recent Exports"),
                exportHistory.length === 0 ? (React.createElement("div", { className: "empty-state" },
                    React.createElement("p", null, "No recent exports"))) : (React.createElement("div", { className: "history-list" }, exportHistory.map((item) => (React.createElement("div", { key: item.id, className: "history-item" },
                    React.createElement("div", { className: "history-icon" },
                        item.format === 'csv' && '📊',
                        item.format === 'json' && '{ }',
                        item.format === 'pdf' && '📄',
                        item.format === 'xlsx' && '📈'),
                    React.createElement("div", { className: "history-content" },
                        React.createElement("div", { className: "history-filename" }, item.filename),
                        React.createElement("div", { className: "history-meta" },
                            formatTimestamp(item.timestamp),
                            item.encrypted && (React.createElement(React.Fragment, null,
                                ' • ',
                                React.createElement("span", { className: "badge badge-warning" }, "Encrypted"))),
                            item.signed && (React.createElement(React.Fragment, null,
                                ' • ',
                                React.createElement("span", { className: "badge badge-success" }, "Signed"))))))))))),
            React.createElement("div", { className: "compliance-notice" },
                React.createElement("h4", null, "Compliance & Security Notice"),
                React.createElement("ul", null,
                    React.createElement("li", null, "Exported audit logs contain sensitive information. Handle with care and follow your organization's data protection policies."),
                    React.createElement("li", null, "Encrypted exports use AES-256 encryption. Store passwords securely using a password manager."),
                    React.createElement("li", null, "Digital signatures ensure authenticity and can be verified using your organization's public key."),
                    React.createElement("li", null, "Hash chains allow verification of audit log integrity. Tampering with any event will break the chain."),
                    React.createElement("li", null, "Retain exports according to your compliance requirements (SOC2, GDPR, HIPAA, etc.)."))))));
};
function FormatOption({ format, label, description, icon, selected, onClick, }) {
    return (React.createElement("button", { className: `format-option ${selected ? 'selected' : ''}`, onClick: onClick },
        React.createElement("div", { className: "format-icon" }, icon),
        React.createElement("div", { className: "format-content" },
            React.createElement("div", { className: "format-label" }, label),
            React.createElement("div", { className: "format-description" }, description)),
        selected && React.createElement("div", { className: "format-check" }, "\u2713")));
}
function formatTimestamp(timestamp) {
    const date = new Date(timestamp);
    return new Intl.DateTimeFormat('en-US', {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
    }).format(date);
}
//# sourceMappingURL=AuditExport.js.map