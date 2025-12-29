import React, { useState, useCallback, useEffect } from 'react';
export const NotificationRules = ({ tenantId, apiUrl = '/api/notifications/rules' }) => {
    const [rules, setRules] = useState([]);
    const [loading, setLoading] = useState(false);
    const [editingRule, setEditingRule] = useState(null);
    const [isModalOpen, setIsModalOpen] = useState(false);
    const fetchRules = useCallback(async () => {
        setLoading(true);
        try {
            const response = await fetch(`${apiUrl}?tenantId=${tenantId}`, {
                credentials: 'include'
            });
            const data = await response.json();
            setRules(data.rules || []);
        }
        catch (err) {
            console.error('Error fetching rules:', err);
        }
        finally {
            setLoading(false);
        }
    }, [apiUrl, tenantId]);
    useEffect(() => {
        fetchRules();
    }, [fetchRules]);
    const handleCreate = useCallback(() => {
        setEditingRule({
            name: '',
            enabled: true,
            priority: 100,
            conditions: [],
            conditionLogic: 'AND',
            actions: []
        });
        setIsModalOpen(true);
    }, []);
    const handleEdit = useCallback((rule) => {
        setEditingRule(rule);
        setIsModalOpen(true);
    }, []);
    const handleSave = useCallback(async () => {
        if (!editingRule)
            return;
        try {
            const method = editingRule.id ? 'PUT' : 'POST';
            const url = editingRule.id ? `${apiUrl}/${editingRule.id}` : apiUrl;
            const response = await fetch(url, {
                method,
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                body: JSON.stringify({ ...editingRule, tenantId })
            });
            if (response.ok) {
                await fetchRules();
                setIsModalOpen(false);
                setEditingRule(null);
            }
        }
        catch (err) {
            console.error('Error saving rule:', err);
            alert('Failed to save rule');
        }
    }, [editingRule, apiUrl, tenantId, fetchRules]);
    const handleDelete = useCallback(async (id) => {
        if (!window.confirm('Are you sure you want to delete this rule?'))
            return;
        try {
            await fetch(`${apiUrl}/${id}`, {
                method: 'DELETE',
                credentials: 'include'
            });
            await fetchRules();
        }
        catch (err) {
            console.error('Error deleting rule:', err);
        }
    }, [apiUrl, fetchRules]);
    const handleToggleEnabled = useCallback(async (rule) => {
        try {
            await fetch(`${apiUrl}/${rule.id}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                body: JSON.stringify({ ...rule, enabled: !rule.enabled })
            });
            await fetchRules();
        }
        catch (err) {
            console.error('Error toggling rule:', err);
        }
    }, [apiUrl, fetchRules]);
    const addCondition = useCallback(() => {
        if (!editingRule)
            return;
        setEditingRule({
            ...editingRule,
            conditions: [
                ...(editingRule.conditions || []),
                { field: 'type', operator: 'eq', value: '' }
            ]
        });
    }, [editingRule]);
    const removeCondition = useCallback((index) => {
        if (!editingRule)
            return;
        setEditingRule({
            ...editingRule,
            conditions: editingRule.conditions?.filter((_, i) => i !== index) || []
        });
    }, [editingRule]);
    const updateCondition = useCallback((index, updates) => {
        if (!editingRule)
            return;
        const newConditions = [...(editingRule.conditions || [])];
        newConditions[index] = { ...newConditions[index], ...updates };
        setEditingRule({ ...editingRule, conditions: newConditions });
    }, [editingRule]);
    const addAction = useCallback(() => {
        if (!editingRule)
            return;
        setEditingRule({
            ...editingRule,
            actions: [
                ...(editingRule.actions || []),
                { type: 'route', config: {} }
            ]
        });
    }, [editingRule]);
    const removeAction = useCallback((index) => {
        if (!editingRule)
            return;
        setEditingRule({
            ...editingRule,
            actions: editingRule.actions?.filter((_, i) => i !== index) || []
        });
    }, [editingRule]);
    const updateAction = useCallback((index, updates) => {
        if (!editingRule)
            return;
        const newActions = [...(editingRule.actions || [])];
        newActions[index] = { ...newActions[index], ...updates };
        setEditingRule({ ...editingRule, actions: newActions });
    }, [editingRule]);
    return (React.createElement("div", { style: { padding: '24px', maxWidth: '1200px', margin: '0 auto' } },
        React.createElement("div", { style: { display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '24px' } },
            React.createElement("div", null,
                React.createElement("h2", { style: { margin: '0 0 4px 0', fontSize: '20px', fontWeight: '600', color: '#111827' } }, "Notification Rules"),
                React.createElement("p", { style: { margin: 0, fontSize: '14px', color: '#6b7280' } }, "Define rules for routing, escalating, and processing notifications")),
            React.createElement("button", { onClick: handleCreate, style: {
                    padding: '10px 20px',
                    fontSize: '14px',
                    fontWeight: '500',
                    border: 'none',
                    borderRadius: '6px',
                    backgroundColor: '#3b82f6',
                    color: '#ffffff',
                    cursor: 'pointer'
                } }, "+ Create Rule")),
        loading ? (React.createElement("div", { style: { padding: '48px', textAlign: 'center', color: '#6b7280' } }, "Loading rules...")) : rules.length === 0 ? (React.createElement("div", { style: { padding: '48px', textAlign: 'center', color: '#6b7280' } },
            React.createElement("div", { style: { fontSize: '48px', marginBottom: '16px' } }, "\u2699\uFE0F"),
            React.createElement("div", { style: { fontSize: '16px', fontWeight: '500', marginBottom: '8px' } }, "No rules configured"),
            React.createElement("div", { style: { fontSize: '14px' } }, "Create rules to automate notification processing"))) : (React.createElement("div", { style: { display: 'grid', gap: '16px' } }, rules.sort((a, b) => b.priority - a.priority).map((rule) => (React.createElement("div", { key: rule.id, style: {
                padding: '16px',
                border: '1px solid #e5e7eb',
                borderRadius: '8px',
                backgroundColor: rule.enabled ? '#ffffff' : '#f9fafb'
            } },
            React.createElement("div", { style: { display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', marginBottom: '12px' } },
                React.createElement("div", null,
                    React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' } },
                        React.createElement("h3", { style: { margin: 0, fontSize: '16px', fontWeight: '600', color: '#111827' } }, rule.name),
                        React.createElement("span", { style: {
                                padding: '2px 8px',
                                fontSize: '11px',
                                fontWeight: '500',
                                borderRadius: '12px',
                                backgroundColor: rule.enabled ? '#dcfce7' : '#f3f4f6',
                                color: rule.enabled ? '#166534' : '#6b7280'
                            } }, rule.enabled ? 'Active' : 'Inactive'),
                        React.createElement("span", { style: {
                                padding: '2px 8px',
                                fontSize: '11px',
                                fontWeight: '500',
                                borderRadius: '12px',
                                backgroundColor: '#e0f2fe',
                                color: '#075985'
                            } },
                            "Priority: ",
                            rule.priority)),
                    rule.description && (React.createElement("p", { style: { margin: '4px 0 0 0', fontSize: '13px', color: '#6b7280' } }, rule.description))),
                React.createElement("div", { style: { display: 'flex', gap: '8px' } },
                    React.createElement("button", { onClick: () => handleToggleEnabled(rule), style: {
                            padding: '6px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#374151',
                            cursor: 'pointer'
                        } }, rule.enabled ? 'Disable' : 'Enable'),
                    React.createElement("button", { onClick: () => handleEdit(rule), style: {
                            padding: '6px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#374151',
                            cursor: 'pointer'
                        } }, "Edit"),
                    React.createElement("button", { onClick: () => handleDelete(rule.id), style: {
                            padding: '6px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: '1px solid #dc2626',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#dc2626',
                            cursor: 'pointer'
                        } }, "Delete"))),
            React.createElement("div", { style: { display: 'grid', gap: '8px' } },
                React.createElement("div", null,
                    React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } },
                        "CONDITIONS (",
                        rule.conditionLogic,
                        ")"),
                    React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '4px' } }, rule.conditions.map((condition, index) => (React.createElement("div", { key: index, style: {
                            padding: '6px 8px',
                            fontSize: '12px',
                            backgroundColor: '#f9fafb',
                            borderRadius: '4px',
                            fontFamily: 'monospace'
                        } },
                        condition.field,
                        " ",
                        condition.operator,
                        " ",
                        JSON.stringify(condition.value)))))),
                React.createElement("div", null,
                    React.createElement("div", { style: { fontSize: '11px', fontWeight: '600', color: '#6b7280', marginBottom: '4px' } },
                        "ACTIONS (",
                        rule.actions.length,
                        ")"),
                    React.createElement("div", { style: { display: 'flex', flexWrap: 'wrap', gap: '4px' } }, rule.actions.map((action, index) => (React.createElement("span", { key: index, style: {
                            padding: '4px 8px',
                            fontSize: '11px',
                            fontWeight: '500',
                            backgroundColor: '#dbeafe',
                            color: '#1e40af',
                            borderRadius: '4px',
                            textTransform: 'capitalize'
                        } }, action.type))))))))))),
        isModalOpen && editingRule && (React.createElement("div", { style: {
                position: 'fixed',
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                backgroundColor: 'rgba(0, 0, 0, 0.5)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                zIndex: 10000
            }, onClick: () => setIsModalOpen(false) },
            React.createElement("div", { onClick: (e) => e.stopPropagation(), style: {
                    backgroundColor: '#ffffff',
                    borderRadius: '8px',
                    padding: '24px',
                    maxWidth: '700px',
                    width: '90%',
                    maxHeight: '80vh',
                    overflowY: 'auto',
                    boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)'
                } },
                React.createElement("h3", { style: { margin: '0 0 24px 0', fontSize: '20px', fontWeight: '600', color: '#111827' } }, editingRule.id ? 'Edit Rule' : 'Create Rule'),
                React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '16px' } },
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Name *"),
                        React.createElement("input", { type: "text", value: editingRule.name, onChange: (e) => setEditingRule({ ...editingRule, name: e.target.value }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Description"),
                        React.createElement("textarea", { value: editingRule.description || '', onChange: (e) => setEditingRule({ ...editingRule, description: e.target.value }), rows: 2, style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px', fontFamily: 'inherit', resize: 'vertical' } })),
                    React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' } },
                        React.createElement("div", null,
                            React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Priority *"),
                            React.createElement("input", { type: "number", value: editingRule.priority, onChange: (e) => setEditingRule({ ...editingRule, priority: parseInt(e.target.value) }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } })),
                        React.createElement("div", null,
                            React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Logic *"),
                            React.createElement("select", { value: editingRule.conditionLogic, onChange: (e) => setEditingRule({ ...editingRule, conditionLogic: e.target.value }), style: { width: '100%', padding: '8px 12px', fontSize: '14px', border: '1px solid #d1d5db', borderRadius: '4px' } },
                                React.createElement("option", { value: "AND" }, "AND"),
                                React.createElement("option", { value: "OR" }, "OR")))),
                    React.createElement("div", null,
                        React.createElement("div", { style: { display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' } },
                            React.createElement("label", { style: { fontSize: '13px', fontWeight: '600', color: '#374151' } }, "Conditions"),
                            React.createElement("button", { onClick: addCondition, style: {
                                    padding: '4px 12px',
                                    fontSize: '12px',
                                    fontWeight: '500',
                                    border: '1px solid #3b82f6',
                                    borderRadius: '4px',
                                    backgroundColor: '#ffffff',
                                    color: '#3b82f6',
                                    cursor: 'pointer'
                                } }, "+ Add Condition")),
                        React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '8px' } }, editingRule.conditions?.map((condition, index) => (React.createElement("div", { key: index, style: { display: 'grid', gridTemplateColumns: '1fr 1fr 1fr auto', gap: '8px', alignItems: 'center' } },
                            React.createElement("input", { type: "text", value: condition.field, onChange: (e) => updateCondition(index, { field: e.target.value }), placeholder: "Field", style: { padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' } }),
                            React.createElement("select", { value: condition.operator, onChange: (e) => updateCondition(index, { operator: e.target.value }), style: { padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' } },
                                React.createElement("option", { value: "eq" }, "Equals"),
                                React.createElement("option", { value: "ne" }, "Not Equals"),
                                React.createElement("option", { value: "gt" }, "Greater Than"),
                                React.createElement("option", { value: "gte" }, "Greater or Equal"),
                                React.createElement("option", { value: "lt" }, "Less Than"),
                                React.createElement("option", { value: "lte" }, "Less or Equal"),
                                React.createElement("option", { value: "in" }, "In"),
                                React.createElement("option", { value: "nin" }, "Not In"),
                                React.createElement("option", { value: "contains" }, "Contains"),
                                React.createElement("option", { value: "matches" }, "Matches")),
                            React.createElement("input", { type: "text", value: typeof condition.value === 'string' ? condition.value : JSON.stringify(condition.value), onChange: (e) => updateCondition(index, { value: e.target.value }), placeholder: "Value", style: { padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' } }),
                            React.createElement("button", { onClick: () => removeCondition(index), style: {
                                    padding: '6px 8px',
                                    fontSize: '12px',
                                    border: 'none',
                                    borderRadius: '4px',
                                    backgroundColor: '#fee2e2',
                                    color: '#dc2626',
                                    cursor: 'pointer'
                                } }, "\u2715")))))),
                    React.createElement("div", null,
                        React.createElement("div", { style: { display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' } },
                            React.createElement("label", { style: { fontSize: '13px', fontWeight: '600', color: '#374151' } }, "Actions"),
                            React.createElement("button", { onClick: addAction, style: {
                                    padding: '4px 12px',
                                    fontSize: '12px',
                                    fontWeight: '500',
                                    border: '1px solid #3b82f6',
                                    borderRadius: '4px',
                                    backgroundColor: '#ffffff',
                                    color: '#3b82f6',
                                    cursor: 'pointer'
                                } }, "+ Add Action")),
                        React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '8px' } }, editingRule.actions?.map((action, index) => (React.createElement("div", { key: index, style: { display: 'flex', gap: '8px', alignItems: 'center' } },
                            React.createElement("select", { value: action.type, onChange: (e) => updateAction(index, { type: e.target.value }), style: { flex: 1, padding: '6px 8px', fontSize: '13px', border: '1px solid #d1d5db', borderRadius: '4px' } },
                                React.createElement("option", { value: "route" }, "Route"),
                                React.createElement("option", { value: "escalate" }, "Escalate"),
                                React.createElement("option", { value: "suppress" }, "Suppress"),
                                React.createElement("option", { value: "transform" }, "Transform"),
                                React.createElement("option", { value: "delay" }, "Delay")),
                            React.createElement("button", { onClick: () => removeAction(index), style: {
                                    padding: '6px 8px',
                                    fontSize: '12px',
                                    border: 'none',
                                    borderRadius: '4px',
                                    backgroundColor: '#fee2e2',
                                    color: '#dc2626',
                                    cursor: 'pointer'
                                } }, "\u2715"))))))),
                React.createElement("div", { style: { display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' } },
                    React.createElement("button", { onClick: () => setIsModalOpen(false), style: {
                            padding: '10px 20px',
                            fontSize: '14px',
                            fontWeight: '500',
                            border: '1px solid #d1d5db',
                            borderRadius: '6px',
                            backgroundColor: '#ffffff',
                            color: '#374151',
                            cursor: 'pointer'
                        } }, "Cancel"),
                    React.createElement("button", { onClick: handleSave, style: {
                            padding: '10px 20px',
                            fontSize: '14px',
                            fontWeight: '500',
                            border: 'none',
                            borderRadius: '6px',
                            backgroundColor: '#3b82f6',
                            color: '#ffffff',
                            cursor: 'pointer'
                        } }, editingRule.id ? 'Update' : 'Create')))))));
};
//# sourceMappingURL=NotificationRules.js.map