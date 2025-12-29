import React, { useState, useCallback, useEffect } from 'react';
import { NotificationType, NotificationPriority, NotificationChannel } from './types';
export const NotificationTemplates = ({ tenantId, apiUrl = '/api/notifications/templates' }) => {
    const [templates, setTemplates] = useState([]);
    const [loading, setLoading] = useState(false);
    const [editingTemplate, setEditingTemplate] = useState(null);
    const [isModalOpen, setIsModalOpen] = useState(false);
    const fetchTemplates = useCallback(async () => {
        setLoading(true);
        try {
            const response = await fetch(`${apiUrl}?tenantId=${tenantId}`, {
                credentials: 'include'
            });
            const data = await response.json();
            setTemplates(data.templates || []);
        }
        catch (err) {
            console.error('Error fetching templates:', err);
        }
        finally {
            setLoading(false);
        }
    }, [apiUrl, tenantId]);
    useEffect(() => {
        fetchTemplates();
    }, [fetchTemplates]);
    const handleCreate = useCallback(() => {
        setEditingTemplate({
            name: '',
            type: NotificationType.INFO,
            priority: NotificationPriority.MEDIUM,
            channels: [NotificationChannel.IN_APP],
            titleTemplate: '',
            messageTemplate: '',
            variables: [],
            active: true
        });
        setIsModalOpen(true);
    }, []);
    const handleEdit = useCallback((template) => {
        setEditingTemplate(template);
        setIsModalOpen(true);
    }, []);
    const handleSave = useCallback(async () => {
        if (!editingTemplate)
            return;
        try {
            const method = editingTemplate.id ? 'PUT' : 'POST';
            const url = editingTemplate.id
                ? `${apiUrl}/${editingTemplate.id}`
                : apiUrl;
            const response = await fetch(url, {
                method,
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                body: JSON.stringify({ ...editingTemplate, tenantId })
            });
            if (response.ok) {
                await fetchTemplates();
                setIsModalOpen(false);
                setEditingTemplate(null);
            }
        }
        catch (err) {
            console.error('Error saving template:', err);
            alert('Failed to save template');
        }
    }, [editingTemplate, apiUrl, tenantId, fetchTemplates]);
    const handleDelete = useCallback(async (id) => {
        if (!window.confirm('Are you sure you want to delete this template?'))
            return;
        try {
            await fetch(`${apiUrl}/${id}`, {
                method: 'DELETE',
                credentials: 'include'
            });
            await fetchTemplates();
        }
        catch (err) {
            console.error('Error deleting template:', err);
        }
    }, [apiUrl, fetchTemplates]);
    const handleToggleActive = useCallback(async (template) => {
        try {
            await fetch(`${apiUrl}/${template.id}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                body: JSON.stringify({ ...template, active: !template.active })
            });
            await fetchTemplates();
        }
        catch (err) {
            console.error('Error toggling template:', err);
        }
    }, [apiUrl, fetchTemplates]);
    return (React.createElement("div", { style: { padding: '24px', maxWidth: '1200px', margin: '0 auto' } },
        React.createElement("div", { style: { display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '24px' } },
            React.createElement("div", null,
                React.createElement("h2", { style: { margin: '0 0 4px 0', fontSize: '20px', fontWeight: '600', color: '#111827' } }, "Notification Templates"),
                React.createElement("p", { style: { margin: 0, fontSize: '14px', color: '#6b7280' } }, "Manage reusable notification templates with dynamic variables")),
            React.createElement("button", { onClick: handleCreate, style: {
                    padding: '10px 20px',
                    fontSize: '14px',
                    fontWeight: '500',
                    border: 'none',
                    borderRadius: '6px',
                    backgroundColor: '#3b82f6',
                    color: '#ffffff',
                    cursor: 'pointer'
                } }, "+ Create Template")),
        loading ? (React.createElement("div", { style: { padding: '48px', textAlign: 'center', color: '#6b7280' } }, "Loading templates...")) : templates.length === 0 ? (React.createElement("div", { style: { padding: '48px', textAlign: 'center', color: '#6b7280' } },
            React.createElement("div", { style: { fontSize: '48px', marginBottom: '16px' } }, "\uD83D\uDCDD"),
            React.createElement("div", { style: { fontSize: '16px', fontWeight: '500', marginBottom: '8px' } }, "No templates yet"),
            React.createElement("div", { style: { fontSize: '14px' } }, "Create your first notification template"))) : (React.createElement("div", { style: { display: 'grid', gap: '16px' } }, templates.map((template) => (React.createElement("div", { key: template.id, style: {
                padding: '16px',
                border: '1px solid #e5e7eb',
                borderRadius: '8px',
                backgroundColor: template.active ? '#ffffff' : '#f9fafb'
            } },
            React.createElement("div", { style: { display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', marginBottom: '12px' } },
                React.createElement("div", { style: { flex: 1 } },
                    React.createElement("div", { style: { display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' } },
                        React.createElement("h3", { style: { margin: 0, fontSize: '16px', fontWeight: '600', color: '#111827' } }, template.name),
                        React.createElement("span", { style: {
                                padding: '2px 8px',
                                fontSize: '11px',
                                fontWeight: '500',
                                borderRadius: '12px',
                                backgroundColor: template.active ? '#dcfce7' : '#f3f4f6',
                                color: template.active ? '#166534' : '#6b7280'
                            } }, template.active ? 'Active' : 'Inactive')),
                    template.description && (React.createElement("p", { style: { margin: '4px 0 0 0', fontSize: '13px', color: '#6b7280' } }, template.description))),
                React.createElement("div", { style: { display: 'flex', gap: '8px' } },
                    React.createElement("button", { onClick: () => handleToggleActive(template), style: {
                            padding: '6px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#374151',
                            cursor: 'pointer'
                        } }, template.active ? 'Deactivate' : 'Activate'),
                    React.createElement("button", { onClick: () => handleEdit(template), style: {
                            padding: '6px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: '1px solid #d1d5db',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#374151',
                            cursor: 'pointer'
                        } }, "Edit"),
                    React.createElement("button", { onClick: () => handleDelete(template.id), style: {
                            padding: '6px 12px',
                            fontSize: '12px',
                            fontWeight: '500',
                            border: '1px solid #dc2626',
                            borderRadius: '4px',
                            backgroundColor: '#ffffff',
                            color: '#dc2626',
                            cursor: 'pointer'
                        } }, "Delete"))),
            React.createElement("div", { style: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px', marginBottom: '12px' } },
                React.createElement("div", null,
                    React.createElement("div", { style: { fontSize: '11px', fontWeight: '500', color: '#6b7280', marginBottom: '4px' } }, "Type"),
                    React.createElement("div", { style: { fontSize: '13px', color: '#111827', textTransform: 'capitalize' } }, template.type)),
                React.createElement("div", null,
                    React.createElement("div", { style: { fontSize: '11px', fontWeight: '500', color: '#6b7280', marginBottom: '4px' } }, "Priority"),
                    React.createElement("div", { style: { fontSize: '13px', color: '#111827', textTransform: 'capitalize' } }, template.priority)),
                React.createElement("div", null,
                    React.createElement("div", { style: { fontSize: '11px', fontWeight: '500', color: '#6b7280', marginBottom: '4px' } }, "Channels"),
                    React.createElement("div", { style: { fontSize: '13px', color: '#111827' } }, template.channels.length)),
                React.createElement("div", null,
                    React.createElement("div", { style: { fontSize: '11px', fontWeight: '500', color: '#6b7280', marginBottom: '4px' } }, "Variables"),
                    React.createElement("div", { style: { fontSize: '13px', color: '#111827' } }, template.variables.length))),
            React.createElement("div", { style: { padding: '12px', backgroundColor: '#f9fafb', borderRadius: '4px', fontSize: '12px', fontFamily: 'monospace' } },
                React.createElement("div", { style: { marginBottom: '8px' } },
                    React.createElement("strong", null, "Title:"),
                    " ",
                    template.titleTemplate),
                React.createElement("div", null,
                    React.createElement("strong", null, "Message:"),
                    " ",
                    template.messageTemplate))))))),
        isModalOpen && editingTemplate && (React.createElement("div", { style: {
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
                    maxWidth: '600px',
                    width: '90%',
                    maxHeight: '80vh',
                    overflowY: 'auto',
                    boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)'
                } },
                React.createElement("h3", { style: { margin: '0 0 24px 0', fontSize: '20px', fontWeight: '600', color: '#111827' } }, editingTemplate.id ? 'Edit Template' : 'Create Template'),
                React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '16px' } },
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Name *"),
                        React.createElement("input", { type: "text", value: editingTemplate.name, onChange: (e) => setEditingTemplate({ ...editingTemplate, name: e.target.value }), style: {
                                width: '100%',
                                padding: '8px 12px',
                                fontSize: '14px',
                                border: '1px solid #d1d5db',
                                borderRadius: '4px'
                            } })),
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Description"),
                        React.createElement("textarea", { value: editingTemplate.description || '', onChange: (e) => setEditingTemplate({ ...editingTemplate, description: e.target.value }), rows: 2, style: {
                                width: '100%',
                                padding: '8px 12px',
                                fontSize: '14px',
                                border: '1px solid #d1d5db',
                                borderRadius: '4px',
                                fontFamily: 'inherit',
                                resize: 'vertical'
                            } })),
                    React.createElement("div", { style: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' } },
                        React.createElement("div", null,
                            React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Type *"),
                            React.createElement("select", { value: editingTemplate.type, onChange: (e) => setEditingTemplate({ ...editingTemplate, type: e.target.value }), style: {
                                    width: '100%',
                                    padding: '8px 12px',
                                    fontSize: '14px',
                                    border: '1px solid #d1d5db',
                                    borderRadius: '4px'
                                } }, Object.values(NotificationType).map(type => (React.createElement("option", { key: type, value: type }, type))))),
                        React.createElement("div", null,
                            React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Priority *"),
                            React.createElement("select", { value: editingTemplate.priority, onChange: (e) => setEditingTemplate({ ...editingTemplate, priority: e.target.value }), style: {
                                    width: '100%',
                                    padding: '8px 12px',
                                    fontSize: '14px',
                                    border: '1px solid #d1d5db',
                                    borderRadius: '4px'
                                } }, Object.values(NotificationPriority).map(priority => (React.createElement("option", { key: priority, value: priority }, priority)))))),
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Title Template *"),
                        React.createElement("input", { type: "text", value: editingTemplate.titleTemplate, onChange: (e) => setEditingTemplate({ ...editingTemplate, titleTemplate: e.target.value }), placeholder: "Use {{variable}} for dynamic content", style: {
                                width: '100%',
                                padding: '8px 12px',
                                fontSize: '14px',
                                border: '1px solid #d1d5db',
                                borderRadius: '4px',
                                fontFamily: 'monospace'
                            } })),
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Message Template *"),
                        React.createElement("textarea", { value: editingTemplate.messageTemplate, onChange: (e) => setEditingTemplate({ ...editingTemplate, messageTemplate: e.target.value }), placeholder: "Use {{variable}} for dynamic content", rows: 3, style: {
                                width: '100%',
                                padding: '8px 12px',
                                fontSize: '14px',
                                border: '1px solid #d1d5db',
                                borderRadius: '4px',
                                fontFamily: 'monospace',
                                resize: 'vertical'
                            } })),
                    React.createElement("div", null,
                        React.createElement("label", { style: { display: 'block', fontSize: '13px', fontWeight: '500', color: '#374151', marginBottom: '4px' } }, "Variables (comma-separated)"),
                        React.createElement("input", { type: "text", value: editingTemplate.variables?.join(', ') || '', onChange: (e) => setEditingTemplate({
                                ...editingTemplate,
                                variables: e.target.value.split(',').map(v => v.trim()).filter(Boolean)
                            }), placeholder: "username, action, timestamp", style: {
                                width: '100%',
                                padding: '8px 12px',
                                fontSize: '14px',
                                border: '1px solid #d1d5db',
                                borderRadius: '4px'
                            } }))),
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
                        } }, editingTemplate.id ? 'Update' : 'Create')))))));
};
//# sourceMappingURL=NotificationTemplates.js.map