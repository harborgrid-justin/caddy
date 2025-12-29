import React, { useState, useCallback } from 'react';
const TRIGGER_TYPES = [
    {
        value: 'schedule',
        label: 'Schedule',
        icon: '⏰',
        description: 'Run on a schedule using cron expressions',
    },
    {
        value: 'webhook',
        label: 'Webhook',
        icon: '🔗',
        description: 'Trigger via HTTP webhook endpoint',
    },
    {
        value: 'data-change',
        label: 'Data Change',
        icon: '💾',
        description: 'Trigger when data changes in database',
    },
    {
        value: 'manual',
        label: 'Manual',
        icon: '👆',
        description: 'Manual execution by user',
    },
    {
        value: 'event',
        label: 'Event',
        icon: '⚡',
        description: 'Trigger on specific events',
    },
];
const CRON_PRESETS = [
    { label: 'Every minute', value: '* * * * *' },
    { label: 'Every 5 minutes', value: '*/5 * * * *' },
    { label: 'Every hour', value: '0 * * * *' },
    { label: 'Every day at midnight', value: '0 0 * * *' },
    { label: 'Every day at 9 AM', value: '0 9 * * *' },
    { label: 'Every Monday at 9 AM', value: '0 9 * * 1' },
    { label: 'First day of month', value: '0 0 1 * *' },
];
const generateId = () => `trigger_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
export const WorkflowTriggers = ({ triggers, onTriggerAdd, onTriggerUpdate, onTriggerDelete, onTriggerToggle, readOnly = false, }) => {
    const [isAdding, setIsAdding] = useState(false);
    const [selectedType, setSelectedType] = useState('schedule');
    const [editingTriggerId, setEditingTriggerId] = useState(null);
    const [newTriggerConfig, setNewTriggerConfig] = useState({});
    const handleAddTrigger = useCallback(() => {
        if (onTriggerAdd) {
            onTriggerAdd({
                type: selectedType,
                config: newTriggerConfig,
                enabled: true,
            });
            setIsAdding(false);
            setNewTriggerConfig({});
        }
    }, [selectedType, newTriggerConfig, onTriggerAdd]);
    const handleUpdateConfig = useCallback((triggerId, key, value) => {
        if (onTriggerUpdate && !readOnly) {
            const trigger = triggers.find((t) => t.id === triggerId);
            if (trigger) {
                onTriggerUpdate(triggerId, {
                    config: {
                        ...trigger.config,
                        [key]: value,
                    },
                });
            }
        }
    }, [triggers, onTriggerUpdate, readOnly]);
    const renderTriggerConfig = useCallback((trigger, isNew = false) => {
        const config = isNew ? newTriggerConfig : trigger.config;
        const updateFn = isNew
            ? (key, value) => setNewTriggerConfig((prev) => ({ ...prev, [key]: value }))
            : (key, value) => handleUpdateConfig(trigger.id, key, value);
        switch (isNew ? selectedType : trigger.type) {
            case 'schedule':
                return (React.createElement(React.Fragment, null,
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Cron Expression"),
                        React.createElement("input", { type: "text", value: config.schedule || '', onChange: (e) => updateFn('schedule', e.target.value), disabled: readOnly, placeholder: "* * * * *", style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                                fontFamily: 'monospace',
                            } })),
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Presets"),
                        React.createElement("select", { onChange: (e) => updateFn('schedule', e.target.value), disabled: readOnly, value: "", style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                                cursor: readOnly ? 'default' : 'pointer',
                            } },
                            React.createElement("option", { value: "" }, "Select a preset..."),
                            CRON_PRESETS.map((preset) => (React.createElement("option", { key: preset.value, value: preset.value },
                                preset.label,
                                " (",
                                preset.value,
                                ")")))))));
            case 'webhook':
                return (React.createElement(React.Fragment, null,
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Webhook URL"),
                        React.createElement("input", { type: "text", value: config.webhookUrl || '', readOnly: true, placeholder: "Generated automatically", style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                                backgroundColor: '#f8fafc',
                            } }),
                        React.createElement("p", { style: {
                                fontSize: '11px',
                                color: '#64748b',
                                marginTop: '4px',
                            } }, "POST requests to this URL will trigger the workflow")),
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
                            React.createElement("input", { type: "checkbox", checked: config.requireAuth || false, onChange: (e) => updateFn('requireAuth', e.target.checked), disabled: readOnly, style: { cursor: readOnly ? 'default' : 'pointer' } }),
                            React.createElement("span", { style: { fontSize: '13px', color: '#1e293b' } }, "Require authentication")))));
            case 'data-change':
                return (React.createElement(React.Fragment, null,
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Table/Collection"),
                        React.createElement("input", { type: "text", value: config.table || '', onChange: (e) => updateFn('table', e.target.value), disabled: readOnly, placeholder: "users", style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                            } })),
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Operation"),
                        React.createElement("select", { value: config.operation || 'any', onChange: (e) => updateFn('operation', e.target.value), disabled: readOnly, style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                                cursor: readOnly ? 'default' : 'pointer',
                            } },
                            React.createElement("option", { value: "any" }, "Any change"),
                            React.createElement("option", { value: "insert" }, "Insert"),
                            React.createElement("option", { value: "update" }, "Update"),
                            React.createElement("option", { value: "delete" }, "Delete"))),
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Filter (SQL WHERE clause)"),
                        React.createElement("input", { type: "text", value: config.filter || '', onChange: (e) => updateFn('filter', e.target.value), disabled: readOnly, placeholder: "status = 'active'", style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                                fontFamily: 'monospace',
                            } }))));
            case 'event':
                return (React.createElement(React.Fragment, null,
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Event Name"),
                        React.createElement("input", { type: "text", value: config.event || '', onChange: (e) => updateFn('event', e.target.value), disabled: readOnly, placeholder: "user.created", style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                            } })),
                    React.createElement("div", { style: { marginBottom: '12px' } },
                        React.createElement("label", { style: {
                                display: 'block',
                                fontSize: '12px',
                                fontWeight: 600,
                                color: '#64748b',
                                marginBottom: '6px',
                            } }, "Event Source"),
                        React.createElement("input", { type: "text", value: config.source || '', onChange: (e) => updateFn('source', e.target.value), disabled: readOnly, placeholder: "Optional event source", style: {
                                width: '100%',
                                padding: '8px 12px',
                                border: '1px solid #e2e8f0',
                                borderRadius: '4px',
                                fontSize: '13px',
                            } }))));
            case 'manual':
                return (React.createElement("div", { style: {
                        padding: '16px',
                        backgroundColor: '#f8fafc',
                        borderRadius: '6px',
                        textAlign: 'center',
                        color: '#64748b',
                        fontSize: '13px',
                    } }, "Manual triggers require no configuration. The workflow can be started manually from the UI or via API."));
            default:
                return null;
        }
    }, [
        newTriggerConfig,
        selectedType,
        handleUpdateConfig,
        readOnly,
    ]);
    const containerStyle = {
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        backgroundColor: '#fff',
        borderRadius: '8px',
        overflow: 'hidden',
        border: '1px solid #e2e8f0',
    };
    return (React.createElement("div", { style: containerStyle },
        React.createElement("div", { style: {
                padding: '16px',
                borderBottom: '1px solid #e2e8f0',
                backgroundColor: '#f8fafc',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
            } },
            React.createElement("h3", { style: { fontSize: '16px', fontWeight: 600 } }, "Workflow Triggers"),
            !readOnly && !isAdding && (React.createElement("button", { onClick: () => setIsAdding(true), style: {
                    padding: '8px 16px',
                    backgroundColor: '#3b82f6',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    fontSize: '13px',
                    fontWeight: 500,
                } }, "+ Add Trigger"))),
        React.createElement("div", { style: { flex: 1, overflow: 'auto', padding: '16px' } },
            isAdding && (React.createElement("div", { style: {
                    padding: '16px',
                    backgroundColor: '#f8fafc',
                    border: '2px dashed #3b82f6',
                    borderRadius: '8px',
                    marginBottom: '16px',
                } },
                React.createElement("h4", { style: { fontSize: '14px', fontWeight: 600, marginBottom: '12px' } }, "Add New Trigger"),
                React.createElement("div", { style: { marginBottom: '16px' } },
                    React.createElement("label", { style: {
                            display: 'block',
                            fontSize: '12px',
                            fontWeight: 600,
                            color: '#64748b',
                            marginBottom: '8px',
                        } }, "Trigger Type"),
                    React.createElement("div", { style: {
                            display: 'grid',
                            gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
                            gap: '8px',
                        } }, TRIGGER_TYPES.map((type) => (React.createElement("button", { key: type.value, onClick: () => setSelectedType(type.value), style: {
                            padding: '12px',
                            backgroundColor: selectedType === type.value ? '#eff6ff' : '#fff',
                            border: `2px solid ${selectedType === type.value ? '#3b82f6' : '#e2e8f0'}`,
                            borderRadius: '6px',
                            cursor: 'pointer',
                            textAlign: 'left',
                            transition: 'all 0.2s ease',
                        } },
                        React.createElement("div", { style: { fontSize: '20px', marginBottom: '4px' } }, type.icon),
                        React.createElement("div", { style: { fontSize: '13px', fontWeight: 600, color: '#1e293b' } }, type.label)))))),
                renderTriggerConfig({ id: '', type: selectedType, config: newTriggerConfig, enabled: true }, true),
                React.createElement("div", { style: { display: 'flex', gap: '8px', justifyContent: 'flex-end' } },
                    React.createElement("button", { onClick: () => {
                            setIsAdding(false);
                            setNewTriggerConfig({});
                        }, style: {
                            padding: '8px 16px',
                            backgroundColor: '#fff',
                            color: '#64748b',
                            border: '1px solid #e2e8f0',
                            borderRadius: '6px',
                            cursor: 'pointer',
                            fontSize: '13px',
                        } }, "Cancel"),
                    React.createElement("button", { onClick: handleAddTrigger, style: {
                            padding: '8px 16px',
                            backgroundColor: '#3b82f6',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '6px',
                            cursor: 'pointer',
                            fontSize: '13px',
                            fontWeight: 500,
                        } }, "Add Trigger")))),
            triggers.length > 0 ? (React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '12px' } }, triggers.map((trigger) => {
                const typeInfo = TRIGGER_TYPES.find((t) => t.value === trigger.type);
                const isEditing = editingTriggerId === trigger.id;
                return (React.createElement("div", { key: trigger.id, style: {
                        padding: '16px',
                        backgroundColor: trigger.enabled ? '#fff' : '#f8fafc',
                        border: '1px solid #e2e8f0',
                        borderRadius: '8px',
                    } },
                    React.createElement("div", { style: {
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'space-between',
                            marginBottom: isEditing ? '12px' : '0',
                        } },
                        React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '12px' } },
                            React.createElement("span", { style: { fontSize: '24px' } }, typeInfo?.icon),
                            React.createElement("div", null,
                                React.createElement("div", { style: { fontSize: '14px', fontWeight: 600, color: '#1e293b' } }, typeInfo?.label),
                                React.createElement("div", { style: { fontSize: '12px', color: '#64748b' } }, typeInfo?.description))),
                        !readOnly && (React.createElement("div", { style: { display: 'flex', gap: '8px', alignItems: 'center' } },
                            React.createElement("label", { style: {
                                    display: 'flex',
                                    alignItems: 'center',
                                    gap: '6px',
                                    cursor: 'pointer',
                                } },
                                React.createElement("input", { type: "checkbox", checked: trigger.enabled, onChange: (e) => onTriggerToggle &&
                                        onTriggerToggle(trigger.id, e.target.checked), style: { cursor: 'pointer' } }),
                                React.createElement("span", { style: { fontSize: '12px', color: '#64748b' } }, "Enabled")),
                            React.createElement("button", { onClick: () => setEditingTriggerId(isEditing ? null : trigger.id), style: {
                                    padding: '4px 12px',
                                    backgroundColor: '#fff',
                                    color: '#64748b',
                                    border: '1px solid #e2e8f0',
                                    borderRadius: '4px',
                                    cursor: 'pointer',
                                    fontSize: '12px',
                                } }, isEditing ? 'Done' : 'Edit'),
                            React.createElement("button", { onClick: () => onTriggerDelete && onTriggerDelete(trigger.id), style: {
                                    padding: '4px 12px',
                                    backgroundColor: '#ef4444',
                                    color: '#fff',
                                    border: 'none',
                                    borderRadius: '4px',
                                    cursor: 'pointer',
                                    fontSize: '12px',
                                } }, "Delete")))),
                    isEditing && React.createElement("div", { style: { marginTop: '12px' } }, renderTriggerConfig(trigger))));
            }))) : (!isAdding && (React.createElement("div", { style: {
                    textAlign: 'center',
                    color: '#94a3b8',
                    padding: '40px 20px',
                } }, "No triggers configured. Add a trigger to start automating your workflow."))))));
};
export default WorkflowTriggers;
//# sourceMappingURL=WorkflowTriggers.js.map