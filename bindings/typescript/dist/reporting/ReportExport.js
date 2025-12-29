import React, { useState, useCallback } from 'react';
export const ReportExport = ({ reportData, onExport, showPreview = true, }) => {
    const [format, setFormat] = useState('pdf');
    const [config, setConfig] = useState(createDefaultConfig('pdf'));
    const [exporting, setExporting] = useState(false);
    const [progress, setProgress] = useState(0);
    const handleFormatChange = useCallback((newFormat) => {
        setFormat(newFormat);
        setConfig(createDefaultConfig(newFormat));
    }, []);
    const updateOptions = useCallback((updates) => {
        setConfig((prev) => ({
            ...prev,
            options: { ...prev.options, ...updates },
        }));
    }, []);
    const updateConfig = useCallback((updates) => {
        setConfig((prev) => ({ ...prev, ...updates }));
    }, []);
    const handleExport = useCallback(async () => {
        setExporting(true);
        setProgress(0);
        try {
            const progressInterval = setInterval(() => {
                setProgress((prev) => Math.min(prev + 10, 90));
            }, 200);
            await onExport(config);
            clearInterval(progressInterval);
            setProgress(100);
            setTimeout(() => {
                setExporting(false);
                setProgress(0);
            }, 1000);
        }
        catch (error) {
            console.error('Export failed:', error);
            alert('Export failed. Please try again.');
            setExporting(false);
            setProgress(0);
        }
    }, [config, onExport]);
    const renderPdfOptions = () => {
        const options = config.options;
        return (React.createElement("div", { style: styles.optionsPanel },
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Page Size"),
                React.createElement("select", { value: options.pageSize, onChange: (e) => updateOptions({ pageSize: e.target.value }), style: styles.select },
                    React.createElement("option", { value: "A4" }, "A4"),
                    React.createElement("option", { value: "Letter" }, "Letter"),
                    React.createElement("option", { value: "Legal" }, "Legal"),
                    React.createElement("option", { value: "A3" }, "A3"),
                    React.createElement("option", { value: "Tabloid" }, "Tabloid"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Orientation"),
                React.createElement("select", { value: options.orientation, onChange: (e) => updateOptions({ orientation: e.target.value }), style: styles.select },
                    React.createElement("option", { value: "portrait" }, "Portrait"),
                    React.createElement("option", { value: "landscape" }, "Landscape"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("h4", { style: styles.subsectionTitle }, "Margins (mm)"),
                React.createElement("div", { style: styles.marginsGrid },
                    React.createElement("div", null,
                        React.createElement("label", { style: styles.smallLabel }, "Top"),
                        React.createElement("input", { type: "number", value: options.margins.top, onChange: (e) => updateOptions({
                                margins: { ...options.margins, top: Number(e.target.value) },
                            }), style: styles.smallInput })),
                    React.createElement("div", null,
                        React.createElement("label", { style: styles.smallLabel }, "Right"),
                        React.createElement("input", { type: "number", value: options.margins.right, onChange: (e) => updateOptions({
                                margins: { ...options.margins, right: Number(e.target.value) },
                            }), style: styles.smallInput })),
                    React.createElement("div", null,
                        React.createElement("label", { style: styles.smallLabel }, "Bottom"),
                        React.createElement("input", { type: "number", value: options.margins.bottom, onChange: (e) => updateOptions({
                                margins: { ...options.margins, bottom: Number(e.target.value) },
                            }), style: styles.smallInput })),
                    React.createElement("div", null,
                        React.createElement("label", { style: styles.smallLabel }, "Left"),
                        React.createElement("input", { type: "number", value: options.margins.left, onChange: (e) => updateOptions({
                                margins: { ...options.margins, left: Number(e.target.value) },
                            }), style: styles.smallInput })))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: options.includeTableOfContents, onChange: (e) => updateOptions({ includeTableOfContents: e.target.checked }) }),
                    React.createElement("span", null, "Include table of contents"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: options.includePageNumbers, onChange: (e) => updateOptions({ includePageNumbers: e.target.checked }) }),
                    React.createElement("span", null, "Include page numbers"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: options.compression, onChange: (e) => updateOptions({ compression: e.target.checked }) }),
                    React.createElement("span", null, "Enable compression")))));
    };
    const renderExcelOptions = () => {
        const options = config.options;
        return (React.createElement("div", { style: styles.optionsPanel },
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Sheet Name"),
                React.createElement("input", { type: "text", value: options.sheetName, onChange: (e) => updateOptions({ sheetName: e.target.value }), style: styles.input, placeholder: "Sheet1" })),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: options.includeCharts, onChange: (e) => updateOptions({ includeCharts: e.target.checked }) }),
                    React.createElement("span", null, "Include charts"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: options.includeFormatting, onChange: (e) => updateOptions({ includeFormatting: e.target.checked }) }),
                    React.createElement("span", null, "Include formatting"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: options.autoFilterHeaders, onChange: (e) => updateOptions({ autoFilterHeaders: e.target.checked }) }),
                    React.createElement("span", null, "Auto-filter headers"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: options.freezeHeader, onChange: (e) => updateOptions({ freezeHeader: e.target.checked }) }),
                    React.createElement("span", null, "Freeze header row"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Password (optional)"),
                React.createElement("input", { type: "password", value: options.password || '', onChange: (e) => updateOptions({ password: e.target.value }), style: styles.input, placeholder: "Leave empty for no password" }))));
    };
    const renderCsvOptions = () => {
        const options = config.options;
        return (React.createElement("div", { style: styles.optionsPanel },
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Delimiter"),
                React.createElement("select", { value: options.delimiter, onChange: (e) => updateOptions({ delimiter: e.target.value }), style: styles.select },
                    React.createElement("option", { value: "," }, "Comma (,)"),
                    React.createElement("option", { value: ";" }, "Semicolon (;)"),
                    React.createElement("option", { value: "\\t" }, "Tab"),
                    React.createElement("option", { value: "|" }, "Pipe (|)"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Quote Character"),
                React.createElement("select", { value: options.quote, onChange: (e) => updateOptions({ quote: e.target.value }), style: styles.select },
                    React.createElement("option", { value: '"' }, "Double Quote (\")"),
                    React.createElement("option", { value: "'" }, "Single Quote (')"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Encoding"),
                React.createElement("select", { value: options.encoding, onChange: (e) => updateOptions({ encoding: e.target.value }), style: styles.select },
                    React.createElement("option", { value: "utf-8" }, "UTF-8"),
                    React.createElement("option", { value: "utf-16" }, "UTF-16"),
                    React.createElement("option", { value: "iso-8859-1" }, "ISO-8859-1"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: options.includeHeader, onChange: (e) => updateOptions({ includeHeader: e.target.checked }) }),
                    React.createElement("span", null, "Include header row"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Line Ending"),
                React.createElement("select", { value: options.lineEnding, onChange: (e) => updateOptions({ lineEnding: e.target.value }), style: styles.select },
                    React.createElement("option", { value: "\\n" }, "LF (\\n) - Unix/Mac"),
                    React.createElement("option", { value: "\\r\\n" }, "CRLF (\\r\\n) - Windows")))));
    };
    const renderPowerPointOptions = () => {
        const options = config.options;
        return (React.createElement("div", { style: styles.optionsPanel },
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Slide Layout"),
                React.createElement("select", { value: options.slideLayout, onChange: (e) => updateOptions({
                        slideLayout: e.target.value,
                    }), style: styles.select },
                    React.createElement("option", { value: "title" }, "Title Slide"),
                    React.createElement("option", { value: "content" }, "Content"),
                    React.createElement("option", { value: "titleAndContent" }, "Title and Content"),
                    React.createElement("option", { value: "blank" }, "Blank"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: options.includeCharts, onChange: (e) => updateOptions({ includeCharts: e.target.checked }) }),
                    React.createElement("span", null, "Include charts"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: options.includeData, onChange: (e) => updateOptions({ includeData: e.target.checked }) }),
                    React.createElement("span", null, "Include data tables"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Theme (optional)"),
                React.createElement("input", { type: "text", value: options.theme || '', onChange: (e) => updateOptions({ theme: e.target.value }), style: styles.input, placeholder: "Default" }))));
    };
    const renderOptions = () => {
        switch (format) {
            case 'pdf':
                return renderPdfOptions();
            case 'excel':
                return renderExcelOptions();
            case 'csv':
                return renderCsvOptions();
            case 'powerpoint':
                return renderPowerPointOptions();
            default:
                return React.createElement("div", null, "JSON export has no additional options");
        }
    };
    return (React.createElement("div", { style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("h3", { style: styles.title }, "Export Configuration")),
        React.createElement("div", { style: styles.content },
            React.createElement("div", { style: styles.section },
                React.createElement("h4", { style: styles.sectionTitle }, "Export Format"),
                React.createElement("div", { style: styles.formatGrid }, exportFormats.map((fmt) => (React.createElement("button", { key: fmt.value, onClick: () => handleFormatChange(fmt.value), style: {
                        ...styles.formatButton,
                        ...(format === fmt.value ? styles.formatButtonActive : {}),
                    } },
                    React.createElement("span", { style: styles.formatIcon }, fmt.icon),
                    React.createElement("span", { style: styles.formatLabel }, fmt.label)))))),
            React.createElement("div", { style: styles.section },
                React.createElement("h4", { style: styles.sectionTitle },
                    format.toUpperCase(),
                    " Options"),
                renderOptions()),
            React.createElement("div", { style: styles.section },
                React.createElement("h4", { style: styles.sectionTitle }, "General Options"),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "File Name"),
                    React.createElement("input", { type: "text", value: config.fileName || '', onChange: (e) => updateConfig({ fileName: e.target.value }), style: styles.input, placeholder: "report-{{date}}" })),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.checkboxLabel },
                        React.createElement("input", { type: "checkbox", checked: !!config.watermark, onChange: (e) => updateConfig({
                                watermark: e.target.checked
                                    ? { text: 'CONFIDENTIAL', opacity: 0.3, position: 'center' }
                                    : undefined,
                            }) }),
                        React.createElement("span", null, "Add watermark"))),
                config.watermark && (React.createElement(React.Fragment, null,
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Watermark Text"),
                        React.createElement("input", { type: "text", value: config.watermark.text, onChange: (e) => updateConfig({
                                watermark: { ...config.watermark, text: e.target.value },
                            }), style: styles.input })),
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Opacity"),
                        React.createElement("input", { type: "range", min: "0", max: "1", step: "0.1", value: config.watermark.opacity, onChange: (e) => updateConfig({
                                watermark: {
                                    ...config.watermark,
                                    opacity: Number(e.target.value),
                                },
                            }), style: styles.slider }),
                        React.createElement("span", { style: styles.sliderValue }, config.watermark.opacity)),
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Position"),
                        React.createElement("select", { value: config.watermark.position, onChange: (e) => updateConfig({
                                watermark: {
                                    ...config.watermark,
                                    position: e.target.value,
                                },
                            }), style: styles.select },
                            React.createElement("option", { value: "center" }, "Center"),
                            React.createElement("option", { value: "corner" }, "Corner")))))),
            reportData && (React.createElement("div", { style: styles.section },
                React.createElement("h4", { style: styles.sectionTitle }, "Export Summary"),
                React.createElement("div", { style: styles.summary },
                    React.createElement("div", { style: styles.summaryItem },
                        React.createElement("span", { style: styles.summaryLabel }, "Rows:"),
                        React.createElement("span", null, reportData.totalRows.toLocaleString())),
                    React.createElement("div", { style: styles.summaryItem },
                        React.createElement("span", { style: styles.summaryLabel }, "Columns:"),
                        React.createElement("span", null, reportData.columns.length)),
                    React.createElement("div", { style: styles.summaryItem },
                        React.createElement("span", { style: styles.summaryLabel }, "Format:"),
                        React.createElement("span", null, format.toUpperCase())),
                    React.createElement("div", { style: styles.summaryItem },
                        React.createElement("span", { style: styles.summaryLabel }, "Estimated Size:"),
                        React.createElement("span", null, estimateFileSize(reportData, format)))))),
            React.createElement("div", { style: styles.exportSection },
                React.createElement("button", { onClick: handleExport, disabled: exporting, style: {
                        ...styles.exportButton,
                        ...(exporting ? styles.exportButtonDisabled : {}),
                    } }, exporting ? `Exporting... ${progress}%` : `Export as ${format.toUpperCase()}`),
                exporting && (React.createElement("div", { style: styles.progressBar },
                    React.createElement("div", { style: { ...styles.progressFill, width: `${progress}%` } })))))));
};
function createDefaultConfig(format) {
    let options;
    switch (format) {
        case 'pdf':
            options = {
                pageSize: 'A4',
                orientation: 'portrait',
                margins: { top: 20, right: 20, bottom: 20, left: 20 },
                includeTableOfContents: false,
                includePageNumbers: true,
                compression: true,
            };
            break;
        case 'excel':
            options = {
                sheetName: 'Report',
                includeCharts: true,
                includeFormatting: true,
                autoFilterHeaders: true,
                freezeHeader: true,
            };
            break;
        case 'csv':
            options = {
                delimiter: ',',
                quote: '"',
                encoding: 'utf-8',
                includeHeader: true,
                lineEnding: '\n',
            };
            break;
        case 'powerpoint':
            options = {
                slideLayout: 'titleAndContent',
                includeCharts: true,
                includeData: true,
            };
            break;
        default:
            options = {};
    }
    return {
        format,
        options,
    };
}
function estimateFileSize(data, format) {
    const baseSize = data.totalRows * data.columns.length * 50;
    let multiplier = 1;
    switch (format) {
        case 'pdf':
            multiplier = 2;
            break;
        case 'excel':
            multiplier = 1.5;
            break;
        case 'csv':
            multiplier = 0.5;
            break;
        case 'powerpoint':
            multiplier = 3;
            break;
        case 'json':
            multiplier = 1.2;
            break;
    }
    const sizeInBytes = baseSize * multiplier;
    const sizeInKB = sizeInBytes / 1024;
    const sizeInMB = sizeInKB / 1024;
    if (sizeInMB > 1) {
        return `${sizeInMB.toFixed(2)} MB`;
    }
    else {
        return `${sizeInKB.toFixed(2)} KB`;
    }
}
const exportFormats = [
    { value: 'pdf', label: 'PDF', icon: '📄' },
    { value: 'excel', label: 'Excel', icon: '📊' },
    { value: 'csv', label: 'CSV', icon: '📝' },
    { value: 'powerpoint', label: 'PowerPoint', icon: '📽️' },
    { value: 'json', label: 'JSON', icon: '{}' },
];
const styles = {
    container: {
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        fontFamily: 'Inter, system-ui, sans-serif',
        overflow: 'hidden',
    },
    header: {
        padding: '12px 16px',
        borderBottom: '1px solid #e2e8f0',
        backgroundColor: '#f8fafc',
    },
    title: {
        fontSize: '14px',
        fontWeight: 600,
        margin: 0,
        color: '#1e293b',
    },
    content: {
        flex: 1,
        overflow: 'auto',
        padding: '16px',
    },
    section: {
        marginBottom: '24px',
        paddingBottom: '24px',
        borderBottom: '1px solid #e2e8f0',
    },
    sectionTitle: {
        fontSize: '13px',
        fontWeight: 600,
        margin: '0 0 12px 0',
        color: '#1e293b',
    },
    formatGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(120px, 1fr))',
        gap: '8px',
    },
    formatButton: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: '6px',
        padding: '16px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        transition: 'all 0.2s',
    },
    formatButtonActive: {
        borderColor: '#2563eb',
        backgroundColor: '#eff6ff',
    },
    formatIcon: {
        fontSize: '32px',
    },
    formatLabel: {
        fontSize: '13px',
        fontWeight: 500,
        color: '#475569',
    },
    optionsPanel: {
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
    },
    formGroup: {
        marginBottom: '12px',
    },
    label: {
        display: 'block',
        fontSize: '12px',
        fontWeight: 500,
        color: '#475569',
        marginBottom: '4px',
    },
    input: {
        width: '100%',
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '13px',
    },
    select: {
        width: '100%',
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '13px',
        cursor: 'pointer',
    },
    checkboxLabel: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        fontSize: '13px',
        color: '#475569',
        cursor: 'pointer',
    },
    subsectionTitle: {
        fontSize: '12px',
        fontWeight: 600,
        margin: '0 0 8px 0',
        color: '#64748b',
    },
    marginsGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(4, 1fr)',
        gap: '8px',
    },
    smallLabel: {
        display: 'block',
        fontSize: '11px',
        fontWeight: 500,
        color: '#64748b',
        marginBottom: '2px',
    },
    smallInput: {
        width: '100%',
        padding: '4px 6px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '12px',
    },
    slider: {
        width: 'calc(100% - 50px)',
        marginRight: '8px',
    },
    sliderValue: {
        fontSize: '12px',
        color: '#64748b',
        fontWeight: 500,
    },
    summary: {
        backgroundColor: '#f8fafc',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        padding: '12px',
    },
    summaryItem: {
        display: 'flex',
        justifyContent: 'space-between',
        padding: '6px 0',
        fontSize: '13px',
        borderBottom: '1px solid #e2e8f0',
    },
    summaryLabel: {
        fontWeight: 600,
        color: '#475569',
    },
    exportSection: {
        marginTop: '24px',
    },
    exportButton: {
        width: '100%',
        padding: '12px',
        border: 'none',
        borderRadius: '6px',
        backgroundColor: '#2563eb',
        color: '#ffffff',
        fontSize: '14px',
        fontWeight: 600,
        cursor: 'pointer',
        transition: 'all 0.2s',
    },
    exportButtonDisabled: {
        backgroundColor: '#94a3b8',
        cursor: 'not-allowed',
    },
    progressBar: {
        width: '100%',
        height: '4px',
        backgroundColor: '#e2e8f0',
        borderRadius: '2px',
        marginTop: '12px',
        overflow: 'hidden',
    },
    progressFill: {
        height: '100%',
        backgroundColor: '#2563eb',
        transition: 'width 0.3s',
    },
};
export default ReportExport;
//# sourceMappingURL=ReportExport.js.map