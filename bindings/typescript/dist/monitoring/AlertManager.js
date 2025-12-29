import React, { useEffect, useState } from 'react';
import { AlertSeverity, MetricType } from './types';
export const AlertManager = ({ service, onAlertClick, className = '' }) => {
    const [rules, setRules] = useState([]);
    const [channels, setChannels] = useState([]);
    const [loading, setLoading] = useState(true);
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [editingRule, setEditingRule] = useState(null);
    const [formData, setFormData] = useState({
        name: '',
        description: '',
        enabled: true,
        severity: AlertSeverity.MEDIUM,
        service: service || '',
        threshold: {
            metric: MetricType.CPU,
            operator: 'gt',
            value: 80,
            duration: 300,
            evaluationWindow: 60
        },
        notificationChannels: [],
        cooldown: 300
    });
    useEffect(() => {
        fetchAlertRules();
        fetchNotificationChannels();
    }, [service]);
    const fetchAlertRules = async () => {
        try {
            setLoading(true);
            const params = service ? `?service=${service}` : '';
            const response = await fetch(`/api/monitoring/alerts/rules${params}`);
            if (!response.ok)
                throw new Error('Failed to fetch alert rules');
            const data = await response.json();
            setRules(data);
        }
        catch (error) {
            console.error('[AlertManager] Failed to fetch rules:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const fetchNotificationChannels = async () => {
        try {
            const response = await fetch('/api/monitoring/notifications/channels');
            if (!response.ok)
                throw new Error('Failed to fetch channels');
            const data = await response.json();
            setChannels(data);
        }
        catch (error) {
            console.error('[AlertManager] Failed to fetch channels:', error);
        }
    };
    const createRule = async () => {
        try {
            const response = await fetch('/api/monitoring/alerts/rules', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    ...formData,
                    createdAt: new Date(),
                    updatedAt: new Date(),
                    createdBy: 'current-user'
                })
            });
            if (!response.ok)
                throw new Error('Failed to create rule');
            const newRule = await response.json();
            setRules(prev => [...prev, newRule]);
            setShowCreateModal(false);
            resetForm();
        }
        catch (error) {
            console.error('[AlertManager] Failed to create rule:', error);
            alert('Failed to create alert rule');
        }
    };
    const updateRule = async (ruleId, updates) => {
        try {
            const response = await fetch(`/api/monitoring/alerts/rules/${ruleId}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    ...updates,
                    updatedAt: new Date()
                })
            });
            if (!response.ok)
                throw new Error('Failed to update rule');
            const updatedRule = await response.json();
            setRules(prev => prev.map(r => r.id === ruleId ? updatedRule : r));
            setEditingRule(null);
            resetForm();
        }
        catch (error) {
            console.error('[AlertManager] Failed to update rule:', error);
            alert('Failed to update alert rule');
        }
    };
    const deleteRule = async (ruleId) => {
        if (!confirm('Are you sure you want to delete this alert rule?'))
            return;
        try {
            const response = await fetch(`/api/monitoring/alerts/rules/${ruleId}`, {
                method: 'DELETE'
            });
            if (!response.ok)
                throw new Error('Failed to delete rule');
            setRules(prev => prev.filter(r => r.id !== ruleId));
        }
        catch (error) {
            console.error('[AlertManager] Failed to delete rule:', error);
            alert('Failed to delete alert rule');
        }
    };
    const toggleRule = async (ruleId, enabled) => {
        await updateRule(ruleId, { enabled });
    };
    const resetForm = () => {
        setFormData({
            name: '',
            description: '',
            enabled: true,
            severity: AlertSeverity.MEDIUM,
            service: service || '',
            threshold: {
                metric: MetricType.CPU,
                operator: 'gt',
                value: 80,
                duration: 300,
                evaluationWindow: 60
            },
            notificationChannels: [],
            cooldown: 300
        });
    };
    const handleSubmit = (e) => {
        e.preventDefault();
        if (!formData.name || !formData.service) {
            alert('Please fill in all required fields');
            return;
        }
        if (editingRule) {
            updateRule(editingRule.id, formData);
        }
        else {
            createRule();
        }
    };
    const getSeverityColor = (severity) => {
        switch (severity) {
            case AlertSeverity.CRITICAL:
                return '#dc2626';
            case AlertSeverity.HIGH:
                return '#f59e0b';
            case AlertSeverity.MEDIUM:
                return '#3b82f6';
            case AlertSeverity.LOW:
                return '#6b7280';
            default:
                return '#6b7280';
        }
    };
    const getOperatorLabel = (operator) => {
        const labels = {
            gt: 'Greater than',
            gte: 'Greater than or equal',
            lt: 'Less than',
            lte: 'Less than or equal',
            eq: 'Equal to',
            neq: 'Not equal to'
        };
        return labels[operator];
    };
    if (loading) {
        return (React.createElement("div", { style: styles.loading },
            React.createElement("div", { style: styles.spinner }),
            React.createElement("p", null, "Loading alert rules...")));
    }
    return (React.createElement("div", { className: `alert-manager ${className}`, style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("div", null,
                React.createElement("h2", { style: styles.title }, "Alert Management"),
                React.createElement("p", { style: styles.subtitle }, "Configure alert rules and notification channels")),
            React.createElement("button", { style: styles.createButton, onClick: () => {
                    setEditingRule(null);
                    resetForm();
                    setShowCreateModal(true);
                } }, "+ Create Alert Rule")),
        React.createElement("div", { style: styles.stats },
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: styles.statValue }, rules.length),
                React.createElement("div", { style: styles.statLabel }, "Total Rules")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: { ...styles.statValue, color: '#10b981' } }, rules.filter(r => r.enabled).length),
                React.createElement("div", { style: styles.statLabel }, "Enabled")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: { ...styles.statValue, color: '#6b7280' } }, rules.filter(r => !r.enabled).length),
                React.createElement("div", { style: styles.statLabel }, "Disabled")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: styles.statValue }, channels.length),
                React.createElement("div", { style: styles.statLabel }, "Channels"))),
        React.createElement("div", { style: styles.section },
            React.createElement("h3", { style: styles.sectionTitle }, "Alert Rules"),
            rules.length === 0 ? (React.createElement("div", { style: styles.emptyState },
                React.createElement("p", null, "No alert rules configured"),
                React.createElement("button", { style: styles.button, onClick: () => setShowCreateModal(true) }, "Create your first alert rule"))) : (React.createElement("div", { style: styles.rulesList }, rules.map(rule => (React.createElement("div", { key: rule.id, style: styles.ruleCard },
                React.createElement("div", { style: styles.ruleHeader },
                    React.createElement("div", { style: styles.ruleTitle },
                        React.createElement("div", { style: {
                                ...styles.severityDot,
                                backgroundColor: getSeverityColor(rule.severity)
                            } }),
                        React.createElement("span", { style: styles.ruleName }, rule.name),
                        React.createElement("span", { style: {
                                ...styles.statusBadge,
                                backgroundColor: rule.enabled ? '#d1fae5' : '#f3f4f6',
                                color: rule.enabled ? '#065f46' : '#6b7280'
                            } }, rule.enabled ? 'Enabled' : 'Disabled')),
                    React.createElement("div", { style: styles.ruleActions },
                        React.createElement("label", { style: styles.toggle },
                            React.createElement("input", { type: "checkbox", checked: rule.enabled, onChange: (e) => toggleRule(rule.id, e.target.checked), style: styles.toggleInput }),
                            React.createElement("span", { style: styles.toggleSlider })),
                        React.createElement("button", { style: styles.iconButton, onClick: () => {
                                setEditingRule(rule);
                                setFormData(rule);
                                setShowCreateModal(true);
                            } }, "\u270E"),
                        React.createElement("button", { style: { ...styles.iconButton, color: '#ef4444' }, onClick: () => deleteRule(rule.id) }, "\uD83D\uDDD1"))),
                rule.description && (React.createElement("p", { style: styles.ruleDescription }, rule.description)),
                React.createElement("div", { style: styles.ruleDetails },
                    React.createElement("div", { style: styles.ruleDetail },
                        React.createElement("strong", null, "Service:"),
                        " ",
                        rule.service),
                    React.createElement("div", { style: styles.ruleDetail },
                        React.createElement("strong", null, "Metric:"),
                        " ",
                        rule.threshold.metric),
                    React.createElement("div", { style: styles.ruleDetail },
                        React.createElement("strong", null, "Condition:"),
                        ' ',
                        getOperatorLabel(rule.threshold.operator),
                        " ",
                        rule.threshold.value),
                    React.createElement("div", { style: styles.ruleDetail },
                        React.createElement("strong", null, "Duration:"),
                        " ",
                        rule.threshold.duration,
                        "s"),
                    React.createElement("div", { style: styles.ruleDetail },
                        React.createElement("strong", null, "Cooldown:"),
                        " ",
                        rule.cooldown,
                        "s")),
                rule.notificationChannels.length > 0 && (React.createElement("div", { style: styles.channels },
                    React.createElement("strong", null, "Notifications:"),
                    ' ',
                    rule.notificationChannels.map(channelId => {
                        const channel = channels.find(c => c.id === channelId);
                        return channel?.name || channelId;
                    }).join(', '))))))))),
        React.createElement("div", { style: styles.section },
            React.createElement("h3", { style: styles.sectionTitle }, "Notification Channels"),
            React.createElement("div", { style: styles.channelsList }, channels.map(channel => (React.createElement("div", { key: channel.id, style: styles.channelCard },
                React.createElement("div", { style: styles.channelHeader },
                    React.createElement("div", { style: styles.channelName },
                        getChannelIcon(channel.type),
                        " ",
                        channel.name),
                    React.createElement("span", { style: {
                            ...styles.statusBadge,
                            backgroundColor: channel.enabled ? '#d1fae5' : '#f3f4f6',
                            color: channel.enabled ? '#065f46' : '#6b7280'
                        } }, channel.enabled ? 'Enabled' : 'Disabled')),
                React.createElement("div", { style: styles.channelType }, channel.type.toUpperCase()),
                channel.services.length > 0 && (React.createElement("div", { style: styles.channelServices },
                    "Services: ",
                    channel.services.join(', ')))))))),
        showCreateModal && (React.createElement("div", { style: styles.modal, onClick: () => setShowCreateModal(false) },
            React.createElement("div", { style: styles.modalContent, onClick: (e) => e.stopPropagation() },
                React.createElement("div", { style: styles.modalHeader },
                    React.createElement("h3", null,
                        editingRule ? 'Edit' : 'Create',
                        " Alert Rule"),
                    React.createElement("button", { style: styles.modalClose, onClick: () => setShowCreateModal(false) }, "\u00D7")),
                React.createElement("form", { onSubmit: handleSubmit, style: styles.form },
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Rule Name *"),
                        React.createElement("input", { type: "text", value: formData.name, onChange: (e) => setFormData({ ...formData, name: e.target.value }), style: styles.input, placeholder: "e.g., High CPU Usage", required: true })),
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Description"),
                        React.createElement("textarea", { value: formData.description, onChange: (e) => setFormData({ ...formData, description: e.target.value }), style: { ...styles.input, minHeight: '80px' }, placeholder: "Optional description..." })),
                    React.createElement("div", { style: styles.formRow },
                        React.createElement("div", { style: styles.formGroup },
                            React.createElement("label", { style: styles.label }, "Service *"),
                            React.createElement("input", { type: "text", value: formData.service, onChange: (e) => setFormData({ ...formData, service: e.target.value }), style: styles.input, placeholder: "Service name", required: true })),
                        React.createElement("div", { style: styles.formGroup },
                            React.createElement("label", { style: styles.label }, "Severity *"),
                            React.createElement("select", { value: formData.severity, onChange: (e) => setFormData({ ...formData, severity: e.target.value }), style: styles.select }, Object.values(AlertSeverity).map(sev => (React.createElement("option", { key: sev, value: sev }, sev.toUpperCase())))))),
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Metric *"),
                        React.createElement("select", { value: formData.threshold?.metric, onChange: (e) => setFormData({
                                ...formData,
                                threshold: { ...formData.threshold, metric: e.target.value }
                            }), style: styles.select }, Object.values(MetricType).map(metric => (React.createElement("option", { key: metric, value: metric }, metric))))),
                    React.createElement("div", { style: styles.formRow },
                        React.createElement("div", { style: styles.formGroup },
                            React.createElement("label", { style: styles.label }, "Operator *"),
                            React.createElement("select", { value: formData.threshold?.operator, onChange: (e) => setFormData({
                                    ...formData,
                                    threshold: { ...formData.threshold, operator: e.target.value }
                                }), style: styles.select },
                                React.createElement("option", { value: "gt" }, "Greater than (>)"),
                                React.createElement("option", { value: "gte" }, "Greater than or equal (>=)"),
                                React.createElement("option", { value: "lt" }, "Less than (<)"),
                                React.createElement("option", { value: "lte" }, "Less than or equal (<=)"),
                                React.createElement("option", { value: "eq" }, "Equal (=)"),
                                React.createElement("option", { value: "neq" }, "Not equal (!=)"))),
                        React.createElement("div", { style: styles.formGroup },
                            React.createElement("label", { style: styles.label }, "Threshold Value *"),
                            React.createElement("input", { type: "number", value: formData.threshold?.value, onChange: (e) => setFormData({
                                    ...formData,
                                    threshold: { ...formData.threshold, value: parseFloat(e.target.value) }
                                }), style: styles.input, required: true }))),
                    React.createElement("div", { style: styles.formRow },
                        React.createElement("div", { style: styles.formGroup },
                            React.createElement("label", { style: styles.label }, "Duration (seconds) *"),
                            React.createElement("input", { type: "number", value: formData.threshold?.duration, onChange: (e) => setFormData({
                                    ...formData,
                                    threshold: { ...formData.threshold, duration: parseInt(e.target.value) }
                                }), style: styles.input, required: true })),
                        React.createElement("div", { style: styles.formGroup },
                            React.createElement("label", { style: styles.label }, "Cooldown (seconds) *"),
                            React.createElement("input", { type: "number", value: formData.cooldown, onChange: (e) => setFormData({ ...formData, cooldown: parseInt(e.target.value) }), style: styles.input, required: true }))),
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Notification Channels"),
                        React.createElement("div", { style: styles.checkboxGroup }, channels.map(channel => (React.createElement("label", { key: channel.id, style: styles.checkbox },
                            React.createElement("input", { type: "checkbox", checked: formData.notificationChannels?.includes(channel.id), onChange: (e) => {
                                    const selected = formData.notificationChannels || [];
                                    setFormData({
                                        ...formData,
                                        notificationChannels: e.target.checked
                                            ? [...selected, channel.id]
                                            : selected.filter(id => id !== channel.id)
                                    });
                                } }),
                            channel.name,
                            " (",
                            channel.type,
                            ")"))))),
                    React.createElement("div", { style: styles.formActions },
                        React.createElement("button", { type: "button", style: styles.cancelButton, onClick: () => setShowCreateModal(false) }, "Cancel"),
                        React.createElement("button", { type: "submit", style: styles.submitButton },
                            editingRule ? 'Update' : 'Create',
                            " Rule"))))))));
};
const getChannelIcon = (type) => {
    switch (type) {
        case 'email':
            return '✉';
        case 'slack':
            return '💬';
        case 'pagerduty':
            return '📟';
        case 'webhook':
            return '🔗';
        case 'sms':
            return '📱';
        default:
            return '🔔';
    }
};
const styles = {
    container: {
        padding: '24px',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'
    },
    loading: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '48px',
        color: '#6b7280'
    },
    spinner: {
        width: '40px',
        height: '40px',
        border: '4px solid #e5e7eb',
        borderTopColor: '#3b82f6',
        borderRadius: '50%',
        animation: 'spin 1s linear infinite'
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'flex-start',
        marginBottom: '24px'
    },
    title: {
        fontSize: '24px',
        fontWeight: 700,
        color: '#111827',
        margin: 0,
        marginBottom: '4px'
    },
    subtitle: {
        fontSize: '14px',
        color: '#6b7280',
        margin: 0
    },
    createButton: {
        padding: '10px 20px',
        backgroundColor: '#3b82f6',
        color: '#fff',
        border: 'none',
        borderRadius: '8px',
        fontSize: '14px',
        fontWeight: 600,
        cursor: 'pointer'
    },
    stats: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
        gap: '16px',
        marginBottom: '32px'
    },
    statCard: {
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px',
        padding: '20px',
        textAlign: 'center'
    },
    statValue: {
        fontSize: '32px',
        fontWeight: 700,
        color: '#111827',
        marginBottom: '4px'
    },
    statLabel: {
        fontSize: '13px',
        color: '#6b7280',
        fontWeight: 500
    },
    section: {
        marginBottom: '32px'
    },
    sectionTitle: {
        fontSize: '18px',
        fontWeight: 600,
        color: '#111827',
        marginBottom: '16px'
    },
    rulesList: {
        display: 'flex',
        flexDirection: 'column',
        gap: '12px'
    },
    ruleCard: {
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px',
        padding: '20px'
    },
    ruleHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '12px'
    },
    ruleTitle: {
        display: 'flex',
        alignItems: 'center',
        gap: '12px'
    },
    severityDot: {
        width: '12px',
        height: '12px',
        borderRadius: '50%'
    },
    ruleName: {
        fontSize: '16px',
        fontWeight: 600,
        color: '#111827'
    },
    statusBadge: {
        fontSize: '11px',
        fontWeight: 600,
        padding: '4px 8px',
        borderRadius: '12px',
        textTransform: 'uppercase'
    },
    ruleActions: {
        display: 'flex',
        alignItems: 'center',
        gap: '12px'
    },
    toggle: {
        position: 'relative',
        display: 'inline-block',
        width: '44px',
        height: '24px'
    },
    toggleInput: {
        opacity: 0,
        width: 0,
        height: 0
    },
    toggleSlider: {
        position: 'absolute',
        cursor: 'pointer',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: '#cbd5e1',
        borderRadius: '24px',
        transition: '0.3s'
    },
    iconButton: {
        background: 'none',
        border: 'none',
        fontSize: '18px',
        cursor: 'pointer',
        color: '#6b7280',
        padding: '4px 8px'
    },
    ruleDescription: {
        fontSize: '14px',
        color: '#6b7280',
        marginBottom: '12px'
    },
    ruleDetails: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
        gap: '12px',
        fontSize: '13px',
        color: '#4b5563'
    },
    ruleDetail: {},
    channels: {
        marginTop: '12px',
        paddingTop: '12px',
        borderTop: '1px solid #e5e7eb',
        fontSize: '13px',
        color: '#4b5563'
    },
    channelsList: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(250px, 1fr))',
        gap: '12px'
    },
    channelCard: {
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px',
        padding: '16px'
    },
    channelHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '8px'
    },
    channelName: {
        fontSize: '15px',
        fontWeight: 600,
        color: '#111827'
    },
    channelType: {
        fontSize: '12px',
        color: '#6b7280',
        marginBottom: '4px'
    },
    channelServices: {
        fontSize: '12px',
        color: '#6b7280'
    },
    emptyState: {
        textAlign: 'center',
        padding: '48px',
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px'
    },
    button: {
        padding: '8px 16px',
        backgroundColor: '#3b82f6',
        color: '#fff',
        border: 'none',
        borderRadius: '6px',
        fontSize: '14px',
        fontWeight: 500,
        cursor: 'pointer',
        marginTop: '12px'
    },
    modal: {
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000
    },
    modalContent: {
        backgroundColor: '#fff',
        borderRadius: '12px',
        maxWidth: '600px',
        width: '90%',
        maxHeight: '90vh',
        overflow: 'auto'
    },
    modalHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '20px',
        borderBottom: '1px solid #e5e7eb'
    },
    modalClose: {
        background: 'none',
        border: 'none',
        fontSize: '32px',
        cursor: 'pointer',
        color: '#6b7280'
    },
    form: {
        padding: '20px'
    },
    formGroup: {
        marginBottom: '16px'
    },
    formRow: {
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        gap: '16px'
    },
    label: {
        display: 'block',
        fontSize: '14px',
        fontWeight: 500,
        color: '#374151',
        marginBottom: '6px'
    },
    input: {
        width: '100%',
        padding: '8px 12px',
        border: '1px solid #d1d5db',
        borderRadius: '6px',
        fontSize: '14px',
        outline: 'none',
        boxSizing: 'border-box'
    },
    select: {
        width: '100%',
        padding: '8px 12px',
        border: '1px solid #d1d5db',
        borderRadius: '6px',
        fontSize: '14px',
        outline: 'none',
        boxSizing: 'border-box',
        backgroundColor: '#fff'
    },
    checkboxGroup: {
        display: 'flex',
        flexDirection: 'column',
        gap: '8px'
    },
    checkbox: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        fontSize: '14px',
        color: '#374151',
        cursor: 'pointer'
    },
    formActions: {
        display: 'flex',
        justifyContent: 'flex-end',
        gap: '12px',
        marginTop: '24px',
        paddingTop: '20px',
        borderTop: '1px solid #e5e7eb'
    },
    cancelButton: {
        padding: '8px 16px',
        backgroundColor: '#fff',
        color: '#374151',
        border: '1px solid #d1d5db',
        borderRadius: '6px',
        fontSize: '14px',
        fontWeight: 500,
        cursor: 'pointer'
    },
    submitButton: {
        padding: '8px 16px',
        backgroundColor: '#3b82f6',
        color: '#fff',
        border: 'none',
        borderRadius: '6px',
        fontSize: '14px',
        fontWeight: 500,
        cursor: 'pointer'
    }
};
export default AlertManager;
//# sourceMappingURL=AlertManager.js.map