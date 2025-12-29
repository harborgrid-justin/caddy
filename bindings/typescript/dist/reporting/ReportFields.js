import React, { useState, useCallback } from 'react';
export const ReportFields = ({ availableTables, selectedFields, onChange, readOnly = false, showAggregations = true, showCalculations = true, }) => {
    const [searchTerm, setSearchTerm] = useState('');
    const [expandedTables, setExpandedTables] = useState(new Set());
    const [editingField, setEditingField] = useState(null);
    const [showCalculationBuilder, setShowCalculationBuilder] = useState(false);
    const toggleTable = useCallback((tableName) => {
        setExpandedTables((prev) => {
            const newSet = new Set(prev);
            if (newSet.has(tableName)) {
                newSet.delete(tableName);
            }
            else {
                newSet.add(tableName);
            }
            return newSet;
        });
    }, []);
    const addField = useCallback((table, field) => {
        if (readOnly)
            return;
        const newField = {
            field: `${table.name}.${field.name}`,
            alias: field.displayName || field.name,
            aggregation: field.defaultAggregation,
            format: field.format,
        };
        onChange([...selectedFields, newField]);
    }, [selectedFields, onChange, readOnly]);
    const removeField = useCallback((index) => {
        if (readOnly)
            return;
        onChange(selectedFields.filter((_, i) => i !== index));
    }, [selectedFields, onChange, readOnly]);
    const updateField = useCallback((index, updates) => {
        if (readOnly)
            return;
        onChange(selectedFields.map((field, i) => (i === index ? { ...field, ...updates } : field)));
    }, [selectedFields, onChange, readOnly]);
    const moveField = useCallback((index, direction) => {
        if (readOnly)
            return;
        const newFields = [...selectedFields];
        const targetIndex = direction === 'up' ? index - 1 : index + 1;
        if (targetIndex < 0 || targetIndex >= newFields.length)
            return;
        [newFields[index], newFields[targetIndex]] = [newFields[targetIndex], newFields[index]];
        onChange(newFields);
    }, [selectedFields, onChange, readOnly]);
    const addCalculatedField = useCallback((calculation) => {
        if (readOnly)
            return;
        const newField = {
            field: 'calculated',
            alias: calculation.alias,
            calculation: {
                expression: calculation.expression,
                fields: calculation.fields,
            },
        };
        onChange([...selectedFields, newField]);
        setShowCalculationBuilder(false);
    }, [selectedFields, onChange, readOnly]);
    const filteredTables = availableTables
        .map((table) => ({
        ...table,
        fields: table.fields.filter((field) => field.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
            (field.displayName?.toLowerCase() || '').includes(searchTerm.toLowerCase())),
    }))
        .filter((table) => table.fields.length > 0);
    const isFieldSelected = useCallback((tableName, fieldName) => {
        return selectedFields.some((sf) => sf.field === `${tableName}.${fieldName}`);
    }, [selectedFields]);
    return (React.createElement("div", { style: styles.container },
        React.createElement("div", { style: styles.availablePanel },
            React.createElement("div", { style: styles.panelHeader },
                React.createElement("h3", { style: styles.panelTitle }, "Available Fields"),
                React.createElement("input", { type: "text", placeholder: "Search fields...", value: searchTerm, onChange: (e) => setSearchTerm(e.target.value), style: styles.searchInput })),
            React.createElement("div", { style: styles.fieldsList },
                filteredTables.map((table) => (React.createElement("div", { key: table.name, style: styles.tableGroup },
                    React.createElement("div", { onClick: () => toggleTable(table.name), style: styles.tableGroupHeader },
                        React.createElement("span", { style: styles.expandIcon }, expandedTables.has(table.name) ? '▼' : '▶'),
                        React.createElement("span", { style: styles.tableGroupName }, table.displayName || table.name),
                        React.createElement("span", { style: styles.fieldCount },
                            "(",
                            table.fields.length,
                            ")")),
                    expandedTables.has(table.name) && (React.createElement("div", { style: styles.tableFields }, table.fields.map((field) => {
                        const selected = isFieldSelected(table.name, field.name);
                        return (React.createElement("div", { key: field.name, onClick: () => !selected && addField(table, field), style: {
                                ...styles.fieldItem,
                                ...(selected ? styles.fieldItemSelected : {}),
                                cursor: selected || readOnly ? 'default' : 'pointer',
                            }, draggable: !readOnly, title: field.description },
                            React.createElement("span", { style: styles.fieldIcon }, getFieldTypeIcon(field.dataType)),
                            React.createElement("span", { style: styles.fieldName }, field.displayName || field.name),
                            React.createElement("span", { style: styles.fieldType }, field.dataType),
                            selected && React.createElement("span", { style: styles.selectedBadge }, "\u2713")));
                    })))))),
                filteredTables.length === 0 && (React.createElement("div", { style: styles.emptyState },
                    React.createElement("div", { style: styles.emptyStateIcon }, "\uD83D\uDD0D"),
                    React.createElement("div", { style: styles.emptyStateText }, "No fields found")))),
            showCalculations && !readOnly && (React.createElement("div", { style: styles.panelFooter },
                React.createElement("button", { onClick: () => setShowCalculationBuilder(true), style: styles.addCalculationButton }, "\u0192 Add Calculated Field")))),
        React.createElement("div", { style: styles.selectedPanel },
            React.createElement("div", { style: styles.panelHeader },
                React.createElement("h3", { style: styles.panelTitle },
                    "Selected Fields (",
                    selectedFields.length,
                    ")"),
                !readOnly && selectedFields.length > 0 && (React.createElement("button", { onClick: () => onChange([]), style: styles.clearButton }, "Clear All"))),
            React.createElement("div", { style: styles.selectedList },
                selectedFields.map((field, index) => (React.createElement("div", { key: index, style: {
                        ...styles.selectedField,
                        ...(editingField === field.field ? styles.selectedFieldEditing : {}),
                    } },
                    React.createElement("div", { style: styles.selectedFieldHeader },
                        React.createElement("div", { style: styles.selectedFieldInfo },
                            React.createElement("span", { style: styles.selectedFieldIcon }, field.calculation ? 'ƒ' : '📊'),
                            React.createElement("span", { style: styles.selectedFieldName }, field.alias || field.field),
                            field.aggregation && (React.createElement("span", { style: styles.aggregationBadge }, field.aggregation.toUpperCase()))),
                        !readOnly && (React.createElement("div", { style: styles.selectedFieldActions },
                            React.createElement("button", { onClick: () => moveField(index, 'up'), disabled: index === 0, style: styles.actionButton, title: "Move Up" }, "\u2191"),
                            React.createElement("button", { onClick: () => moveField(index, 'down'), disabled: index === selectedFields.length - 1, style: styles.actionButton, title: "Move Down" }, "\u2193"),
                            React.createElement("button", { onClick: () => setEditingField(editingField === field.field ? null : field.field), style: styles.actionButton, title: "Edit" }, "\u270E"),
                            React.createElement("button", { onClick: () => removeField(index), style: styles.deleteButton, title: "Remove" }, "\u2715")))),
                    editingField === field.field && (React.createElement("div", { style: styles.fieldEditor },
                        React.createElement("div", { style: styles.editorRow },
                            React.createElement("label", { style: styles.editorLabel }, "Alias"),
                            React.createElement("input", { type: "text", value: field.alias || '', onChange: (e) => updateField(index, { alias: e.target.value }), style: styles.editorInput, placeholder: "Display name" })),
                        showAggregations && !field.calculation && (React.createElement("div", { style: styles.editorRow },
                            React.createElement("label", { style: styles.editorLabel }, "Aggregation"),
                            React.createElement("select", { value: field.aggregation || '', onChange: (e) => updateField(index, {
                                    aggregation: e.target.value || undefined,
                                }), style: styles.editorSelect },
                                React.createElement("option", { value: "" }, "None"),
                                React.createElement("option", { value: "sum" }, "Sum"),
                                React.createElement("option", { value: "avg" }, "Average"),
                                React.createElement("option", { value: "count" }, "Count"),
                                React.createElement("option", { value: "min" }, "Minimum"),
                                React.createElement("option", { value: "max" }, "Maximum"),
                                React.createElement("option", { value: "distinct" }, "Distinct Count"),
                                React.createElement("option", { value: "median" }, "Median")))),
                        React.createElement("div", { style: styles.editorRow },
                            React.createElement("label", { style: styles.editorLabel }, "Format"),
                            React.createElement("input", { type: "text", value: field.format || '', onChange: (e) => updateField(index, { format: e.target.value }), style: styles.editorInput, placeholder: "e.g., #,##0.00 or $#,##0" })),
                        field.calculation && (React.createElement("div", { style: styles.editorRow },
                            React.createElement("label", { style: styles.editorLabel }, "Expression"),
                            React.createElement("input", { type: "text", value: field.calculation.expression, readOnly: true, style: { ...styles.editorInput, backgroundColor: '#f8fafc' } })))))))),
                selectedFields.length === 0 && (React.createElement("div", { style: styles.emptyState },
                    React.createElement("div", { style: styles.emptyStateIcon }, "\uD83D\uDCCA"),
                    React.createElement("div", { style: styles.emptyStateText }, "No fields selected"),
                    React.createElement("div", { style: styles.emptyStateHint }, "Click fields from the left panel to add them"))))),
        showCalculationBuilder && (React.createElement(CalculationBuilder, { availableTables: availableTables, selectedFields: selectedFields, onSave: addCalculatedField, onCancel: () => setShowCalculationBuilder(false) }))));
};
const CalculationBuilder = ({ availableTables, selectedFields, onSave, onCancel, }) => {
    const [alias, setAlias] = useState('');
    const [expression, setExpression] = useState('');
    const [selectedFieldsForCalc, setSelectedFieldsForCalc] = useState([]);
    const handleSave = () => {
        if (!alias || !expression) {
            alert('Please provide both alias and expression');
            return;
        }
        onSave({ expression, fields: selectedFieldsForCalc, alias });
    };
    const insertFunction = (func) => {
        setExpression((prev) => prev + func + '()');
    };
    const insertField = (fieldPath) => {
        setExpression((prev) => prev + `{${fieldPath}}`);
        if (!selectedFieldsForCalc.includes(fieldPath)) {
            setSelectedFieldsForCalc((prev) => [...prev, fieldPath]);
        }
    };
    return (React.createElement("div", { style: styles.modalOverlay },
        React.createElement("div", { style: styles.modalContent },
            React.createElement("div", { style: styles.modalHeader },
                React.createElement("h3", { style: styles.modalTitle }, "Calculated Field Builder"),
                React.createElement("button", { onClick: onCancel, style: styles.modalCloseButton }, "\u2715")),
            React.createElement("div", { style: styles.modalBody },
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Field Name"),
                    React.createElement("input", { type: "text", value: alias, onChange: (e) => setAlias(e.target.value), style: styles.input, placeholder: "e.g., Total Revenue" })),
                React.createElement("div", { style: styles.formGroup },
                    React.createElement("label", { style: styles.label }, "Expression"),
                    React.createElement("textarea", { value: expression, onChange: (e) => setExpression(e.target.value), style: styles.textarea, placeholder: "e.g., {sales.quantity} * {sales.price}", rows: 4 })),
                React.createElement("div", { style: styles.functionsPanel },
                    React.createElement("div", { style: styles.functionsPanelTitle }, "Functions"),
                    React.createElement("div", { style: styles.functionsGrid }, ['SUM', 'AVG', 'MIN', 'MAX', 'COUNT', 'ROUND', 'ABS', 'SQRT'].map((func) => (React.createElement("button", { key: func, onClick: () => insertFunction(func), style: styles.functionButton }, func))))),
                React.createElement("div", { style: styles.fieldsPanel },
                    React.createElement("div", { style: styles.fieldsPanelTitle }, "Available Fields"),
                    React.createElement("div", { style: styles.availableFieldsList }, selectedFields.map((field, index) => (React.createElement("button", { key: index, onClick: () => insertField(field.field), style: styles.fieldButton }, field.alias || field.field)))))),
            React.createElement("div", { style: styles.modalFooter },
                React.createElement("button", { onClick: onCancel, style: styles.cancelButton }, "Cancel"),
                React.createElement("button", { onClick: handleSave, style: styles.saveButton }, "Add Field")))));
};
function getFieldTypeIcon(dataType) {
    switch (dataType) {
        case 'string':
            return '📝';
        case 'number':
            return '🔢';
        case 'date':
            return '📅';
        case 'boolean':
            return '☑️';
        case 'json':
            return '{}';
        case 'array':
            return '[]';
        default:
            return '•';
    }
}
const styles = {
    container: {
        display: 'flex',
        height: '100%',
        gap: '16px',
        fontFamily: 'Inter, system-ui, sans-serif',
    },
    availablePanel: {
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        overflow: 'hidden',
    },
    selectedPanel: {
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        overflow: 'hidden',
    },
    panelHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '12px 16px',
        borderBottom: '1px solid #e2e8f0',
        backgroundColor: '#f8fafc',
    },
    panelTitle: {
        fontSize: '14px',
        fontWeight: 600,
        margin: 0,
        color: '#1e293b',
    },
    searchInput: {
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '12px',
        width: '200px',
    },
    clearButton: {
        padding: '4px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '12px',
        color: '#ef4444',
    },
    fieldsList: {
        flex: 1,
        overflow: 'auto',
        padding: '8px',
    },
    tableGroup: {
        marginBottom: '8px',
    },
    tableGroupHeader: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        padding: '8px',
        cursor: 'pointer',
        borderRadius: '4px',
        transition: 'background-color 0.2s',
    },
    expandIcon: {
        fontSize: '10px',
        color: '#64748b',
    },
    tableGroupName: {
        flex: 1,
        fontSize: '13px',
        fontWeight: 600,
        color: '#1e293b',
    },
    fieldCount: {
        fontSize: '11px',
        color: '#94a3b8',
    },
    tableFields: {
        paddingLeft: '16px',
    },
    fieldItem: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        padding: '6px 8px',
        borderRadius: '4px',
        marginBottom: '2px',
        transition: 'all 0.2s',
    },
    fieldItemSelected: {
        backgroundColor: '#eff6ff',
        borderLeft: '2px solid #2563eb',
    },
    fieldIcon: {
        fontSize: '14px',
    },
    fieldName: {
        flex: 1,
        fontSize: '12px',
        color: '#475569',
    },
    fieldType: {
        fontSize: '10px',
        color: '#94a3b8',
        textTransform: 'uppercase',
    },
    selectedBadge: {
        color: '#10b981',
        fontSize: '12px',
    },
    panelFooter: {
        padding: '12px 16px',
        borderTop: '1px solid #e2e8f0',
    },
    addCalculationButton: {
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
    selectedList: {
        flex: 1,
        overflow: 'auto',
        padding: '8px',
    },
    selectedField: {
        marginBottom: '8px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        backgroundColor: '#ffffff',
        transition: 'all 0.2s',
    },
    selectedFieldEditing: {
        borderColor: '#2563eb',
        boxShadow: '0 0 0 3px rgba(37, 99, 235, 0.1)',
    },
    selectedFieldHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '10px 12px',
    },
    selectedFieldInfo: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        flex: 1,
    },
    selectedFieldIcon: {
        fontSize: '16px',
    },
    selectedFieldName: {
        fontSize: '13px',
        fontWeight: 500,
        color: '#1e293b',
    },
    aggregationBadge: {
        fontSize: '10px',
        padding: '2px 6px',
        backgroundColor: '#dbeafe',
        color: '#1e40af',
        borderRadius: '4px',
        fontWeight: 600,
    },
    selectedFieldActions: {
        display: 'flex',
        gap: '4px',
    },
    actionButton: {
        padding: '4px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '12px',
    },
    deleteButton: {
        padding: '4px 8px',
        border: '1px solid #fee2e2',
        borderRadius: '4px',
        backgroundColor: '#fef2f2',
        color: '#ef4444',
        cursor: 'pointer',
        fontSize: '12px',
    },
    fieldEditor: {
        padding: '12px',
        borderTop: '1px solid #e2e8f0',
        backgroundColor: '#f8fafc',
    },
    editorRow: {
        marginBottom: '12px',
    },
    editorLabel: {
        display: 'block',
        fontSize: '11px',
        fontWeight: 600,
        color: '#475569',
        marginBottom: '4px',
    },
    editorInput: {
        width: '100%',
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '12px',
    },
    editorSelect: {
        width: '100%',
        padding: '6px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        fontSize: '12px',
        cursor: 'pointer',
    },
    emptyState: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '48px 16px',
        gap: '8px',
    },
    emptyStateIcon: {
        fontSize: '32px',
    },
    emptyStateText: {
        fontSize: '13px',
        color: '#64748b',
        fontWeight: 500,
    },
    emptyStateHint: {
        fontSize: '11px',
        color: '#94a3b8',
    },
    modalOverlay: {
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
    },
    modalContent: {
        backgroundColor: '#ffffff',
        borderRadius: '8px',
        width: '600px',
        maxHeight: '80vh',
        display: 'flex',
        flexDirection: 'column',
        boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1)',
    },
    modalHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '16px',
        borderBottom: '1px solid #e2e8f0',
    },
    modalTitle: {
        fontSize: '16px',
        fontWeight: 600,
        margin: 0,
        color: '#1e293b',
    },
    modalCloseButton: {
        border: 'none',
        background: 'none',
        fontSize: '20px',
        color: '#64748b',
        cursor: 'pointer',
    },
    modalBody: {
        flex: 1,
        overflow: 'auto',
        padding: '16px',
    },
    formGroup: {
        marginBottom: '16px',
    },
    label: {
        display: 'block',
        fontSize: '12px',
        fontWeight: 600,
        color: '#475569',
        marginBottom: '6px',
    },
    input: {
        width: '100%',
        padding: '8px 12px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        fontSize: '14px',
    },
    textarea: {
        width: '100%',
        padding: '8px 12px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        fontSize: '14px',
        fontFamily: 'monospace',
        resize: 'vertical',
    },
    functionsPanel: {
        marginBottom: '16px',
    },
    functionsPanelTitle: {
        fontSize: '12px',
        fontWeight: 600,
        color: '#475569',
        marginBottom: '8px',
    },
    functionsGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(4, 1fr)',
        gap: '4px',
    },
    functionButton: {
        padding: '6px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '11px',
        fontWeight: 500,
        color: '#2563eb',
    },
    fieldsPanel: {
        marginBottom: '16px',
    },
    fieldsPanelTitle: {
        fontSize: '12px',
        fontWeight: 600,
        color: '#475569',
        marginBottom: '8px',
    },
    availableFieldsList: {
        display: 'flex',
        flexWrap: 'wrap',
        gap: '4px',
    },
    fieldButton: {
        padding: '4px 8px',
        border: '1px solid #e2e8f0',
        borderRadius: '4px',
        backgroundColor: '#f8fafc',
        cursor: 'pointer',
        fontSize: '11px',
    },
    modalFooter: {
        display: 'flex',
        justifyContent: 'flex-end',
        gap: '8px',
        padding: '16px',
        borderTop: '1px solid #e2e8f0',
    },
    cancelButton: {
        padding: '8px 16px',
        border: '1px solid #e2e8f0',
        borderRadius: '6px',
        backgroundColor: '#ffffff',
        cursor: 'pointer',
        fontSize: '14px',
    },
    saveButton: {
        padding: '8px 16px',
        border: 'none',
        borderRadius: '6px',
        backgroundColor: '#2563eb',
        color: '#ffffff',
        cursor: 'pointer',
        fontSize: '14px',
        fontWeight: 500,
    },
};
export default ReportFields;
//# sourceMappingURL=ReportFields.js.map