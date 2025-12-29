import React, { useState, useCallback } from 'react';
export const ReportCharts = ({ config, availableFields, onChange, readOnly = false, showPreview = true, }) => {
    const [activeTab, setActiveTab] = useState('basic');
    const updateConfig = useCallback((updates) => {
        if (readOnly)
            return;
        onChange({ ...config, ...updates });
    }, [config, onChange, readOnly]);
    const updateOptions = useCallback((updates) => {
        updateConfig({
            options: { ...config.options, ...updates },
        });
    }, [config.options, updateConfig]);
    const updateDataMapping = useCallback((updates) => {
        updateConfig({
            dataMapping: { ...config.dataMapping, ...updates },
        });
    }, [config.dataMapping, updateConfig]);
    const renderBasicTab = () => (React.createElement("div", { style: styles.tabContent },
        React.createElement("div", { style: styles.section },
            React.createElement("h4", { style: styles.sectionTitle }, "Chart Type"),
            React.createElement("div", { style: styles.chartTypeGrid }, chartTypes.map((type) => (React.createElement("button", { key: type.value, onClick: () => updateConfig({ type: type.value }), style: {
                    ...styles.chartTypeButton,
                    ...(config.type === type.value ? styles.chartTypeButtonActive : {}),
                }, disabled: readOnly },
                React.createElement("span", { style: styles.chartTypeIcon }, type.icon),
                React.createElement("span", { style: styles.chartTypeLabel }, type.label)))))),
        React.createElement("div", { style: styles.section },
            React.createElement("h4", { style: styles.sectionTitle }, "Data Mapping"),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "X-Axis Fields"),
                React.createElement("select", { multiple: true, value: config.dataMapping.xAxis, onChange: (e) => {
                        const selected = Array.from(e.target.selectedOptions, (o) => o.value);
                        updateDataMapping({ xAxis: selected });
                    }, style: styles.multiSelect, disabled: readOnly }, availableFields.map((field, index) => (React.createElement("option", { key: index, value: field.field }, field.alias || field.field))))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Y-Axis Fields"),
                React.createElement("select", { multiple: true, value: config.dataMapping.yAxis, onChange: (e) => {
                        const selected = Array.from(e.target.selectedOptions, (o) => o.value);
                        updateDataMapping({ yAxis: selected });
                    }, style: styles.multiSelect, disabled: readOnly }, availableFields.map((field, index) => (React.createElement("option", { key: index, value: field.field }, field.alias || field.field))))),
            (config.type === 'pie' || config.type === 'gauge') && (React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Value Field"),
                React.createElement("select", { value: config.dataMapping.value || '', onChange: (e) => updateDataMapping({ value: e.target.value }), style: styles.select, disabled: readOnly },
                    React.createElement("option", { value: "" }, "Select field..."),
                    availableFields.map((field, index) => (React.createElement("option", { key: index, value: field.field }, field.alias || field.field)))))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Series Field (Optional)"),
                React.createElement("select", { value: config.dataMapping.series || '', onChange: (e) => updateDataMapping({ series: e.target.value || undefined }), style: styles.select, disabled: readOnly },
                    React.createElement("option", { value: "" }, "None"),
                    availableFields.map((field, index) => (React.createElement("option", { key: index, value: field.field }, field.alias || field.field)))))),
        React.createElement("div", { style: styles.section },
            React.createElement("h4", { style: styles.sectionTitle }, "Titles"),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Chart Title"),
                React.createElement("input", { type: "text", value: config.options.title || '', onChange: (e) => updateOptions({ title: e.target.value }), style: styles.input, placeholder: "Chart title", disabled: readOnly })),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Subtitle"),
                React.createElement("input", { type: "text", value: config.options.subtitle || '', onChange: (e) => updateOptions({ subtitle: e.target.value }), style: styles.input, placeholder: "Chart subtitle", disabled: readOnly })))));
    const renderAdvancedTab = () => (React.createElement("div", { style: styles.tabContent },
        React.createElement("div", { style: styles.section },
            React.createElement("h4", { style: styles.sectionTitle }, "Legend"),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: config.options.legend?.show ?? true, onChange: (e) => updateOptions({
                            legend: { ...config.options.legend, show: e.target.checked },
                        }), disabled: readOnly }),
                    React.createElement("span", null, "Show Legend"))),
            config.options.legend?.show !== false && (React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Legend Position"),
                React.createElement("select", { value: config.options.legend?.position || 'top', onChange: (e) => updateOptions({
                        legend: {
                            ...config.options.legend,
                            position: e.target.value,
                        },
                    }), style: styles.select, disabled: readOnly },
                    React.createElement("option", { value: "top" }, "Top"),
                    React.createElement("option", { value: "bottom" }, "Bottom"),
                    React.createElement("option", { value: "left" }, "Left"),
                    React.createElement("option", { value: "right" }, "Right"))))),
        React.createElement("div", { style: styles.section },
            React.createElement("h4", { style: styles.sectionTitle }, "Tooltip"),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: config.options.tooltip?.enabled ?? true, onChange: (e) => updateOptions({
                            tooltip: { ...config.options.tooltip, enabled: e.target.checked },
                        }), disabled: readOnly }),
                    React.createElement("span", null, "Show Tooltip"))),
            config.options.tooltip?.enabled !== false && (React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Tooltip Format"),
                React.createElement("input", { type: "text", value: config.options.tooltip?.format || '', onChange: (e) => updateOptions({
                        tooltip: { ...config.options.tooltip, format: e.target.value },
                    }), style: styles.input, placeholder: "e.g., {b}: {c}", disabled: readOnly })))),
        React.createElement("div", { style: styles.section },
            React.createElement("h4", { style: styles.sectionTitle }, "Axes"),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "X-Axis Label"),
                React.createElement("input", { type: "text", value: config.options.axis?.x?.label || '', onChange: (e) => updateOptions({
                        axis: {
                            ...config.options.axis,
                            x: { ...config.options.axis?.x, label: e.target.value },
                        },
                    }), style: styles.input, placeholder: "X-axis label", disabled: readOnly })),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Y-Axis Label"),
                React.createElement("input", { type: "text", value: config.options.axis?.y?.label || '', onChange: (e) => updateOptions({
                        axis: {
                            ...config.options.axis,
                            y: { ...config.options.axis?.y, label: e.target.value },
                        },
                    }), style: styles.input, placeholder: "Y-axis label", disabled: readOnly })),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: config.options.axis?.x?.grid ?? true, onChange: (e) => updateOptions({
                            axis: {
                                ...config.options.axis,
                                x: { ...config.options.axis?.x, grid: e.target.checked },
                            },
                        }), disabled: readOnly }),
                    React.createElement("span", null, "Show Grid Lines")))),
        React.createElement("div", { style: styles.section },
            React.createElement("h4", { style: styles.sectionTitle }, "Visual Options"),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: config.options.stacked ?? false, onChange: (e) => updateOptions({ stacked: e.target.checked }), disabled: readOnly }),
                    React.createElement("span", null, "Stacked Chart"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: config.options.smooth ?? false, onChange: (e) => updateOptions({ smooth: e.target.checked }), disabled: readOnly }),
                    React.createElement("span", null, "Smooth Lines"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: config.options.animation ?? true, onChange: (e) => updateOptions({ animation: e.target.checked }), disabled: readOnly }),
                    React.createElement("span", null, "Enable Animation"))),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.label }, "Color Palette"),
                React.createElement("div", { style: styles.colorPalette }, (config.options.colors || defaultColors).map((color, index) => (React.createElement("input", { key: index, type: "color", value: color, onChange: (e) => {
                        const newColors = [...(config.options.colors || defaultColors)];
                        newColors[index] = e.target.value;
                        updateOptions({ colors: newColors });
                    }, style: styles.colorInput, disabled: readOnly }))))))));
    const renderDrillDownTab = () => (React.createElement("div", { style: styles.tabContent },
        React.createElement("div", { style: styles.section },
            React.createElement("h4", { style: styles.sectionTitle }, "Drill-Down Configuration"),
            React.createElement("div", { style: styles.formGroup },
                React.createElement("label", { style: styles.checkboxLabel },
                    React.createElement("input", { type: "checkbox", checked: config.drillDown?.enabled ?? false, onChange: (e) => updateConfig({
                            drillDown: {
                                ...config.drillDown,
                                enabled: e.target.checked,
                                levels: config.drillDown?.levels || [],
                            },
                        }), disabled: readOnly }),
                    React.createElement("span", null, "Enable Drill-Down"))),
            config.drillDown?.enabled && (React.createElement(React.Fragment, null,
                React.createElement("div", { style: styles.drillDownLevels }, (config.drillDown.levels || []).map((level, index) => (React.createElement("div", { key: index, style: styles.drillDownLevel },
                    React.createElement("div", { style: styles.drillDownLevelHeader },
                        React.createElement("span", null,
                            "Level ",
                            index + 1),
                        !readOnly && (React.createElement("button", { onClick: () => {
                                const newLevels = config.drillDown.levels.filter((_, i) => i !== index);
                                updateConfig({
                                    drillDown: { ...config.drillDown, levels: newLevels },
                                });
                            }, style: styles.removeButton }, "\u2715"))),
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Field"),
                        React.createElement("select", { value: level.field, onChange: (e) => {
                                const newLevels = [...config.drillDown.levels];
                                newLevels[index] = { ...level, field: e.target.value };
                                updateConfig({
                                    drillDown: { ...config.drillDown, levels: newLevels },
                                });
                            }, style: styles.select, disabled: readOnly },
                            React.createElement("option", { value: "" }, "Select field..."),
                            availableFields.map((field, fieldIndex) => (React.createElement("option", { key: fieldIndex, value: field.field }, field.alias || field.field))))),
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Target Report ID (Optional)"),
                        React.createElement("input", { type: "text", value: level.reportId || '', onChange: (e) => {
                                const newLevels = [...config.drillDown.levels];
                                newLevels[index] = { ...level, reportId: e.target.value };
                                updateConfig({
                                    drillDown: { ...config.drillDown, levels: newLevels },
                                });
                            }, style: styles.input, placeholder: "Report ID", disabled: readOnly })))))),
                !readOnly && (React.createElement("button", { onClick: () => {
                        const newLevel = { field: '', filters: [] };
                        updateConfig({
                            drillDown: {
                                ...config.drillDown,
                                levels: [...(config.drillDown.levels || []), newLevel],
                            },
                        });
                    }, style: styles.addLevelButton }, "+ Add Drill-Down Level")))))));
    return (React.createElement("div", { style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("h3", { style: styles.title }, "Chart Configuration")),
        React.createElement("div", { style: styles.tabs },
            React.createElement("button", { onClick: () => setActiveTab('basic'), style: {
                    ...styles.tab,
                    ...(activeTab === 'basic' ? styles.tabActive : {}),
                } }, "Basic"),
            React.createElement("button", { onClick: () => setActiveTab('advanced'), style: {
                    ...styles.tab,
                    ...(activeTab === 'advanced' ? styles.tabActive : {}),
                } }, "Advanced"),
            React.createElement("button", { onClick: () => setActiveTab('drilldown'), style: {
                    ...styles.tab,
                    ...(activeTab === 'drilldown' ? styles.tabActive : {}),
                } }, "Drill-Down")),
        React.createElement("div", { style: styles.content },
            activeTab === 'basic' && renderBasicTab(),
            activeTab === 'advanced' && renderAdvancedTab(),
            activeTab === 'drilldown' && renderDrillDownTab()),
        showPreview && (React.createElement("div", { style: styles.preview },
            React.createElement("div", { style: styles.previewHeader }, "Preview"),
            React.createElement("div", { style: styles.previewContent },
                React.createElement("div", { style: styles.previewPlaceholder },
                    React.createElement("span", { style: styles.previewIcon }, getChartIcon(config.type)),
                    React.createElement("span", { style: styles.previewText },
                        config.type,
                        " Chart Preview")))))));
};
const chartTypes = [
    { value: 'line', label: 'Line', icon: '📈' },
    { value: 'bar', label: 'Bar', icon: '📊' },
    { value: 'pie', label: 'Pie', icon: '🥧' },
    { value: 'scatter', label: 'Scatter', icon: '⚫' },
    { value: 'area', label: 'Area', icon: '📉' },
    { value: 'heatmap', label: 'Heatmap', icon: '🔥' },
    { value: 'gauge', label: 'Gauge', icon: '🎯' },
    { value: 'funnel', label: 'Funnel', icon: '🔻' },
    { value: 'waterfall', label: 'Waterfall', icon: '💧' },
];
const defaultColors = [
    '#2563eb',
    '#10b981',
    '#f59e0b',
    '#ef4444',
    '#8b5cf6',
    '#06b6d4',
];
function getChartIcon(type) {
    const chartType = chartTypes.find((ct) => ct.value === type);
    return chartType?.icon || '📊';
}
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
    tabs: {
        display: 'flex',
        borderBottom: '1px solid #e2e8f0',
        backgroundColor: '#f8fafc',
    },
    tab: {
        flex: 1,
        padding: '10px 16px',
        border: 'none',
        backgroundColor: 'transparent',
        cursor: 'pointer',
        fontSize: '13px',
        fontWeight: 500,
        color: '#64748b',
        borderBottom: '2px solid transparent',
        transition: 'all 0.2s',
    },
    tabActive: {
        color: '#2563eb',
        borderBottomColor: '#2563eb',
    },
    content: {
        flex: 1,
        overflow: 'auto',
        padding: '16px',
    },
    tabContent: {
        display: 'flex',
        flexDirection: 'column',
        gap: '16px',
    },
    section: {
        marginBottom: '16px',
    },
    sectionTitle: {
        fontSize: '13px',
        fontWeight: 600,
        margin: '0 0 12px 0',
        color: '#1e293b',
    },
    chartTypeGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(3, 1fr)',
        gap: '8px',
    },
    chartTypeButton: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: '4px',
        padding: '12px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        transition: 'all 0.2s',
    },
    chartTypeButtonActive: {
        borderColor: '#2563eb',
        backgroundColor: '#eff6ff',
    },
    chartTypeIcon: {
        fontSize: '24px',
    },
    chartTypeLabel: {
        fontSize: '12px',
        fontWeight: 500,
        color: '#475569',
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
    multiSelect: {
        width: '100%',
        minHeight: '80px',
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '13px',
    },
    checkboxLabel: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        fontSize: '13px',
        color: '#475569',
        cursor: 'pointer',
    },
    colorPalette: {
        display: 'flex',
        gap: '8px',
        flexWrap: 'wrap',
    },
    colorInput: {
        width: '40px',
        height: '40px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        cursor: 'pointer',
    },
    drillDownLevels: {
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
        marginBottom: '12px',
    },
    drillDownLevel: {
        padding: '12px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        backgroundColor: '#f8fafc',
    },
    drillDownLevelHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '12px',
        fontSize: '12px',
        fontWeight: 600,
        color: '#1e293b',
    },
    removeButton: {
        border: 'none',
        background: 'none',
        color: '#ef4444',
        cursor: 'pointer',
        fontSize: '16px',
    },
    addLevelButton: {
        width: '100%',
        padding: '8px',
        border: '1px dashed #2563eb',
        borderRadius: '6px',
        backgroundColor: '#eff6ff',
        color: '#2563eb',
        cursor: 'pointer',
        fontSize: '13px',
        fontWeight: 500,
    },
    preview: {
        borderTop: '1px solid #e2e8f0',
    },
    previewHeader: {
        padding: '8px 16px',
        fontSize: '12px',
        fontWeight: 600,
        color: '#64748b',
        backgroundColor: '#f8fafc',
    },
    previewContent: {
        padding: '16px',
        minHeight: '200px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
    },
    previewPlaceholder: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: '8px',
        color: '#94a3b8',
    },
    previewIcon: {
        fontSize: '48px',
    },
    previewText: {
        fontSize: '13px',
    },
};
export default ReportCharts;
//# sourceMappingURL=ReportCharts.js.map