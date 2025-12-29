import React, { useState, useCallback, useRef } from 'react';
import { useDashboard } from './DashboardLayout';
export const DashboardExport = ({ dashboardConfig, data, availableMetrics = [], availableCharts = [], onExportComplete, onExportError, className = '', }) => {
    const [exportConfig, setExportConfig] = useState({
        format: 'pdf',
        includeCharts: true,
        includeRawData: false,
        includeHeader: true,
        includePageNumbers: true,
        orientation: 'portrait',
        paperSize: 'letter',
        fileName: `dashboard-export-${Date.now()}`,
    });
    const [isExporting, setIsExporting] = useState(false);
    const [exportProgress, setExportProgress] = useState(0);
    const [showAdvanced, setShowAdvanced] = useState(false);
    const exportFormRef = useRef(null);
    const { theme, accessibility } = useDashboard();
    const handleConfigChange = useCallback((updates) => {
        setExportConfig((prev) => ({ ...prev, ...updates }));
    }, []);
    const exportToPDF = useCallback(async () => {
        setExportProgress(25);
        await delay(500);
        const pdfContent = generatePDFContent(exportConfig, dashboardConfig, data);
        setExportProgress(75);
        await delay(500);
        const blob = new Blob([pdfContent], { type: 'application/pdf' });
        setExportProgress(100);
        return blob;
    }, [exportConfig, dashboardConfig, data]);
    const exportToExcel = useCallback(async () => {
        setExportProgress(25);
        await delay(300);
        const excelContent = generateExcelContent(exportConfig, dashboardConfig, data);
        setExportProgress(75);
        await delay(300);
        const blob = new Blob([excelContent], {
            type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
        });
        setExportProgress(100);
        return blob;
    }, [exportConfig, dashboardConfig, data]);
    const exportToCSV = useCallback(async () => {
        setExportProgress(50);
        await delay(200);
        const csvContent = generateCSVContent(exportConfig, dashboardConfig, data);
        setExportProgress(100);
        const blob = new Blob([csvContent], { type: 'text/csv' });
        return blob;
    }, [exportConfig, dashboardConfig, data]);
    const exportToJSON = useCallback(async () => {
        setExportProgress(50);
        await delay(200);
        const jsonContent = JSON.stringify({
            config: dashboardConfig,
            data,
            exportedAt: new Date().toISOString(),
            format: 'json',
        }, null, 2);
        setExportProgress(100);
        const blob = new Blob([jsonContent], { type: 'application/json' });
        return blob;
    }, [dashboardConfig, data]);
    const handleExport = useCallback(async () => {
        setIsExporting(true);
        setExportProgress(0);
        try {
            let blob;
            switch (exportConfig.format) {
                case 'pdf':
                    blob = await exportToPDF();
                    break;
                case 'excel':
                    blob = await exportToExcel();
                    break;
                case 'csv':
                    blob = await exportToCSV();
                    break;
                case 'json':
                    blob = await exportToJSON();
                    break;
                default:
                    throw new Error(`Unsupported export format: ${exportConfig.format}`);
            }
            downloadBlob(blob, getFileName(exportConfig));
            if (onExportComplete) {
                onExportComplete(exportConfig.format, blob);
            }
        }
        catch (error) {
            console.error('Export failed:', error);
            if (onExportError) {
                onExportError(error);
            }
        }
        finally {
            setIsExporting(false);
            setExportProgress(0);
        }
    }, [exportConfig, exportToPDF, exportToExcel, exportToCSV, exportToJSON, onExportComplete, onExportError]);
    const handleEmailExport = useCallback(async () => {
        alert('Email export functionality would be implemented here');
    }, []);
    const handleScheduleExport = useCallback(() => {
        alert('Schedule export functionality would be implemented here');
    }, []);
    return (React.createElement("div", { className: `dashboard-export ${className}`, style: styles.container, role: "region", "aria-label": "Dashboard export options" },
        React.createElement("form", { ref: exportFormRef, style: styles.form },
            React.createElement("div", { style: styles.header },
                React.createElement("h3", { style: styles.title }, "Export Dashboard"),
                React.createElement("p", { style: styles.subtitle }, "Download your dashboard data in various formats")),
            React.createElement("div", { style: styles.section },
                React.createElement("label", { style: styles.label, id: "format-label" }, "Export Format"),
                React.createElement("div", { style: styles.formatGrid, role: "radiogroup", "aria-labelledby": "format-label" },
                    React.createElement(FormatOption, { format: "pdf", icon: "\uD83D\uDCC4", label: "PDF", description: "Formatted report", selected: exportConfig.format === 'pdf', onClick: () => handleConfigChange({ format: 'pdf' }) }),
                    React.createElement(FormatOption, { format: "excel", icon: "\uD83D\uDCCA", label: "Excel", description: "Spreadsheet data", selected: exportConfig.format === 'excel', onClick: () => handleConfigChange({ format: 'excel' }) }),
                    React.createElement(FormatOption, { format: "csv", icon: "\uD83D\uDCCB", label: "CSV", description: "Raw data", selected: exportConfig.format === 'csv', onClick: () => handleConfigChange({ format: 'csv' }) }),
                    React.createElement(FormatOption, { format: "json", icon: "{ }", label: "JSON", description: "API format", selected: exportConfig.format === 'json', onClick: () => handleConfigChange({ format: 'json' }) }))),
            React.createElement("div", { style: styles.section },
                React.createElement("label", { style: styles.label, htmlFor: "file-name" }, "File Name"),
                React.createElement("input", { id: "file-name", type: "text", value: exportConfig.fileName || '', onChange: (e) => handleConfigChange({ fileName: e.target.value }), style: styles.input, placeholder: "Enter file name..." })),
            React.createElement("div", { style: styles.section },
                React.createElement("label", { style: styles.label }, "Include"),
                React.createElement("div", { style: styles.checkboxGroup },
                    React.createElement("label", { style: styles.checkboxLabel },
                        React.createElement("input", { type: "checkbox", checked: exportConfig.includeCharts || false, onChange: (e) => handleConfigChange({ includeCharts: e.target.checked }), style: styles.checkbox }),
                        React.createElement("span", null, "Charts and visualizations")),
                    React.createElement("label", { style: styles.checkboxLabel },
                        React.createElement("input", { type: "checkbox", checked: exportConfig.includeRawData || false, onChange: (e) => handleConfigChange({ includeRawData: e.target.checked }), style: styles.checkbox }),
                        React.createElement("span", null, "Raw data tables")),
                    exportConfig.format === 'pdf' && (React.createElement(React.Fragment, null,
                        React.createElement("label", { style: styles.checkboxLabel },
                            React.createElement("input", { type: "checkbox", checked: exportConfig.includeHeader || false, onChange: (e) => handleConfigChange({ includeHeader: e.target.checked }), style: styles.checkbox }),
                            React.createElement("span", null, "Header and footer")),
                        React.createElement("label", { style: styles.checkboxLabel },
                            React.createElement("input", { type: "checkbox", checked: exportConfig.includePageNumbers || false, onChange: (e) => handleConfigChange({ includePageNumbers: e.target.checked }), style: styles.checkbox }),
                            React.createElement("span", null, "Page numbers")))))),
            exportConfig.format === 'pdf' && (React.createElement("div", { style: styles.section },
                React.createElement("label", { style: styles.label }, "PDF Settings"),
                React.createElement("div", { style: styles.row },
                    React.createElement("div", { style: styles.column },
                        React.createElement("label", { style: styles.smallLabel, htmlFor: "orientation" }, "Orientation"),
                        React.createElement("select", { id: "orientation", value: exportConfig.orientation || 'portrait', onChange: (e) => handleConfigChange({ orientation: e.target.value }), style: styles.select },
                            React.createElement("option", { value: "portrait" }, "Portrait"),
                            React.createElement("option", { value: "landscape" }, "Landscape"))),
                    React.createElement("div", { style: styles.column },
                        React.createElement("label", { style: styles.smallLabel, htmlFor: "paper-size" }, "Paper Size"),
                        React.createElement("select", { id: "paper-size", value: exportConfig.paperSize || 'letter', onChange: (e) => handleConfigChange({ paperSize: e.target.value }), style: styles.select },
                            React.createElement("option", { value: "letter" }, "Letter"),
                            React.createElement("option", { value: "a4" }, "A4"),
                            React.createElement("option", { value: "legal" }, "Legal")))))),
            React.createElement("button", { type: "button", onClick: () => setShowAdvanced(!showAdvanced), style: styles.advancedToggle, "aria-expanded": showAdvanced },
                showAdvanced ? '▼' : '▶',
                " Advanced Options"),
            showAdvanced && (React.createElement("div", { style: styles.advancedSection },
                exportConfig.dateRange !== undefined && (React.createElement("div", { style: styles.section },
                    React.createElement("label", { style: styles.label }, "Date Range"),
                    React.createElement("div", { style: styles.row },
                        React.createElement("input", { type: "date", value: exportConfig.dateRange?.start || '', onChange: (e) => handleConfigChange({
                                dateRange: {
                                    ...exportConfig.dateRange,
                                    start: e.target.value,
                                    end: exportConfig.dateRange?.end || '',
                                },
                            }), style: styles.input, "aria-label": "Start date" }),
                        React.createElement("span", { style: styles.dateSeparator }, "to"),
                        React.createElement("input", { type: "date", value: exportConfig.dateRange?.end || '', onChange: (e) => handleConfigChange({
                                dateRange: {
                                    start: exportConfig.dateRange?.start || '',
                                    end: e.target.value,
                                },
                            }), style: styles.input, "aria-label": "End date" })))),
                React.createElement("div", { style: styles.section },
                    React.createElement("label", { style: styles.label }, "Branding"),
                    React.createElement("input", { type: "text", placeholder: "Company name", value: exportConfig.branding?.companyName || '', onChange: (e) => handleConfigChange({
                            branding: {
                                ...exportConfig.branding,
                                companyName: e.target.value,
                            },
                        }), style: styles.input }),
                    React.createElement("input", { type: "text", placeholder: "Footer text", value: exportConfig.branding?.footer || '', onChange: (e) => handleConfigChange({
                            branding: {
                                ...exportConfig.branding,
                                footer: e.target.value,
                            },
                        }), style: { ...styles.input, marginTop: 8 } })))),
            isExporting && (React.createElement("div", { style: styles.progressSection },
                React.createElement("div", { style: styles.progressBar },
                    React.createElement("div", { style: { ...styles.progressFill, width: `${exportProgress}%` }, role: "progressbar", "aria-valuenow": exportProgress, "aria-valuemin": 0, "aria-valuemax": 100 })),
                React.createElement("p", { style: styles.progressText },
                    "Exporting... ",
                    exportProgress,
                    "%"))),
            React.createElement("div", { style: styles.actions },
                React.createElement("button", { type: "button", onClick: handleExport, disabled: isExporting, style: {
                        ...styles.primaryButton,
                        ...(isExporting && styles.buttonDisabled),
                    }, "aria-label": "Export dashboard" }, isExporting ? 'Exporting...' : `Export as ${exportConfig.format.toUpperCase()}`),
                React.createElement("button", { type: "button", onClick: handleEmailExport, disabled: isExporting, style: styles.secondaryButton, "aria-label": "Email export" }, "\uD83D\uDCE7 Email"),
                React.createElement("button", { type: "button", onClick: handleScheduleExport, disabled: isExporting, style: styles.secondaryButton, "aria-label": "Schedule export" }, "\u23F0 Schedule")))));
};
const FormatOption = ({ format, icon, label, description, selected, onClick, }) => {
    return (React.createElement("button", { type: "button", onClick: onClick, style: {
            ...styles.formatOption,
            ...(selected && styles.formatOptionSelected),
        }, role: "radio", "aria-checked": selected, "aria-label": `${label}: ${description}` },
        React.createElement("div", { style: styles.formatIcon }, icon),
        React.createElement("div", { style: styles.formatLabel }, label),
        React.createElement("div", { style: styles.formatDescription }, description)));
};
function generatePDFContent(config, dashboardConfig, data) {
    return `%PDF-1.4
Dashboard Export - ${dashboardConfig.title}
Generated: ${new Date().toISOString()}
Format: PDF
Configuration: ${JSON.stringify(config)}
`;
}
function generateExcelContent(config, dashboardConfig, data) {
    return `Dashboard Export\n${dashboardConfig.title}\n\nGenerated: ${new Date().toISOString()}\n`;
}
function generateCSVContent(config, dashboardConfig, data) {
    let csv = 'Dashboard Export\n';
    csv += `Title,${dashboardConfig.title}\n`;
    csv += `Generated,${new Date().toISOString()}\n\n`;
    csv += 'Metric,Value,Change,Trend\n';
    if (data && Array.isArray(data.metrics)) {
        data.metrics.forEach((metric) => {
            csv += `${metric.name},${metric.value},${metric.change || 'N/A'},${metric.trend}\n`;
        });
    }
    return csv;
}
function downloadBlob(blob, fileName) {
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = fileName;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
}
function getFileName(config) {
    const ext = {
        pdf: 'pdf',
        excel: 'xlsx',
        csv: 'csv',
        json: 'json',
    }[config.format];
    return `${config.fileName || 'dashboard-export'}.${ext}`;
}
function delay(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}
const styles = {
    container: {
        backgroundColor: 'var(--color-surface, #fff)',
        borderRadius: 8,
        border: '1px solid var(--color-border, #e0e0e0)',
        padding: 24,
    },
    form: {
        display: 'flex',
        flexDirection: 'column',
        gap: 24,
    },
    header: {
        marginBottom: 8,
    },
    title: {
        margin: '0 0 8px 0',
        fontSize: 20,
        fontWeight: 600,
        color: 'var(--color-text, #333)',
    },
    subtitle: {
        margin: 0,
        fontSize: 14,
        color: 'var(--color-text-secondary, #666)',
    },
    section: {
        display: 'flex',
        flexDirection: 'column',
        gap: 12,
    },
    label: {
        fontSize: 14,
        fontWeight: 600,
        color: 'var(--color-text, #333)',
    },
    smallLabel: {
        fontSize: 13,
        fontWeight: 500,
        color: 'var(--color-text-secondary, #666)',
        marginBottom: 4,
    },
    formatGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
        gap: 12,
    },
    formatOption: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        padding: 16,
        border: '2px solid var(--color-border, #e0e0e0)',
        borderRadius: 8,
        backgroundColor: 'var(--color-surface, #fff)',
        cursor: 'pointer',
        transition: 'all var(--animation-duration, 200ms)',
    },
    formatOptionSelected: {
        borderColor: 'var(--color-primary, #1976d2)',
        backgroundColor: '#e3f2fd',
    },
    formatIcon: {
        fontSize: 32,
        marginBottom: 8,
    },
    formatLabel: {
        fontSize: 14,
        fontWeight: 600,
        color: 'var(--color-text, #333)',
        marginBottom: 4,
    },
    formatDescription: {
        fontSize: 12,
        color: 'var(--color-text-secondary, #666)',
        textAlign: 'center',
    },
    input: {
        padding: '10px 12px',
        border: '1px solid var(--color-border, #e0e0e0)',
        borderRadius: 4,
        fontSize: 14,
        backgroundColor: 'var(--color-surface, #fff)',
        color: 'var(--color-text, #333)',
    },
    select: {
        padding: '10px 12px',
        border: '1px solid var(--color-border, #e0e0e0)',
        borderRadius: 4,
        fontSize: 14,
        backgroundColor: 'var(--color-surface, #fff)',
        color: 'var(--color-text, #333)',
        cursor: 'pointer',
    },
    checkboxGroup: {
        display: 'flex',
        flexDirection: 'column',
        gap: 10,
    },
    checkboxLabel: {
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        fontSize: 14,
        color: 'var(--color-text, #333)',
        cursor: 'pointer',
    },
    checkbox: {
        width: 18,
        height: 18,
        cursor: 'pointer',
    },
    row: {
        display: 'flex',
        gap: 12,
    },
    column: {
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
    },
    dateSeparator: {
        display: 'flex',
        alignItems: 'center',
        fontSize: 14,
        color: 'var(--color-text-secondary, #666)',
    },
    advancedToggle: {
        padding: '8px 0',
        border: 'none',
        backgroundColor: 'transparent',
        color: 'var(--color-primary, #1976d2)',
        cursor: 'pointer',
        fontSize: 14,
        fontWeight: 500,
        textAlign: 'left',
    },
    advancedSection: {
        paddingLeft: 16,
        borderLeft: '3px solid var(--color-divider, #e0e0e0)',
        display: 'flex',
        flexDirection: 'column',
        gap: 20,
    },
    progressSection: {
        padding: '16px 0',
    },
    progressBar: {
        width: '100%',
        height: 8,
        backgroundColor: 'var(--color-background, #f5f5f5)',
        borderRadius: 4,
        overflow: 'hidden',
    },
    progressFill: {
        height: '100%',
        backgroundColor: 'var(--color-primary, #1976d2)',
        transition: 'width 300ms ease',
    },
    progressText: {
        marginTop: 8,
        fontSize: 13,
        color: 'var(--color-text-secondary, #666)',
        textAlign: 'center',
    },
    actions: {
        display: 'flex',
        gap: 12,
        flexWrap: 'wrap',
    },
    primaryButton: {
        flex: 1,
        minWidth: 150,
        padding: '12px 24px',
        border: 'none',
        borderRadius: 4,
        backgroundColor: 'var(--color-primary, #1976d2)',
        color: '#fff',
        fontSize: 15,
        fontWeight: 600,
        cursor: 'pointer',
        transition: 'background-color var(--animation-duration, 200ms)',
    },
    secondaryButton: {
        padding: '12px 20px',
        border: '1px solid var(--color-border, #e0e0e0)',
        borderRadius: 4,
        backgroundColor: 'var(--color-surface, #fff)',
        color: 'var(--color-text, #333)',
        fontSize: 14,
        fontWeight: 500,
        cursor: 'pointer',
        transition: 'background-color var(--animation-duration, 200ms)',
    },
    buttonDisabled: {
        opacity: 0.6,
        cursor: 'not-allowed',
    },
};
export default DashboardExport;
//# sourceMappingURL=DashboardExport.js.map