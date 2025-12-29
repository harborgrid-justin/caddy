import React, { useState, useEffect } from 'react';
export const AuditAlerts = ({ organizationId }) => {
    const [alerts, setAlerts] = useState([]);
    const [loading, setLoading] = useState(true);
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [editingAlert, setEditingAlert] = useState(null);
    useEffect(() => {
        loadAlerts();
    }, [organizationId]);
    const loadAlerts = async () => {
        setLoading(true);
        try {
            const params = new URLSearchParams(organizationId ? { organization_id: organizationId } : {});
            const response = await fetch(`/api/audit/alerts?${params}`);
            const data = await response.json();
            setAlerts(data.alerts || []);
        }
        catch (error) {
            console.error('Failed to load alerts:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const handleToggleAlert = async (alert) => {
        try {
            const response = await fetch(`/api/audit/alerts/${alert.id}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ enabled: !alert.enabled }),
            });
            const updated = await response.json();
            setAlerts((prev) => prev.map((a) => (a.id === alert.id ? updated : a)));
        }
        catch (error) {
            console.error('Failed to toggle alert:', error);
        }
    };
    const handleDeleteAlert = async (alertId) => {
        if (!confirm('Are you sure you want to delete this alert?'))
            return;
        try {
            await fetch(`/api/audit/alerts/${alertId}`, {
                method: 'DELETE',
            });
            setAlerts((prev) => prev.filter((a) => a.id !== alertId));
        }
        catch (error) {
            console.error('Failed to delete alert:', error);
        }
    };
    const handleEditAlert = (alert) => {
        setEditingAlert(alert);
        setShowCreateModal(true);
    };
    if (loading) {
        return (React.createElement("div", { className: "audit-alerts loading" },
            React.createElement("div", { className: "loading-spinner" }),
            React.createElement("p", null, "Loading audit alerts...")));
    }
    return (React.createElement("div", { className: "audit-alerts" },
        React.createElement("div", { className: "alerts-header" },
            React.createElement("div", null,
                React.createElement("h2", null, "Audit Alerts"),
                React.createElement("p", { className: "subtitle" }, "Configure alerts for suspicious activity and security events")),
            React.createElement("button", { className: "btn btn-primary", onClick: () => {
                    setEditingAlert(null);
                    setShowCreateModal(true);
                } }, "Create Alert")),
        React.createElement("div", { className: "alert-templates" },
            React.createElement("h3", null, "Quick Templates"),
            React.createElement("div", { className: "templates-grid" },
                React.createElement(AlertTemplate, { title: "Failed Login Attempts", description: "Alert on multiple failed login attempts from same IP", icon: "\uD83D\uDD12", onClick: () => {
                    } }),
                React.createElement(AlertTemplate, { title: "Data Exfiltration", description: "Alert on unusual data export patterns", icon: "\uD83D\uDCE4", onClick: () => {
                    } }),
                React.createElement(AlertTemplate, { title: "Privilege Escalation", description: "Alert on role or permission changes", icon: "\u2B06\uFE0F", onClick: () => {
                    } }),
                React.createElement(AlertTemplate, { title: "Anomalous Behavior", description: "Alert on detected anomalies with high confidence", icon: "\u26A0\uFE0F", onClick: () => {
                    } }))),
        React.createElement("div", { className: "active-alerts" },
            React.createElement("h3", null, "Configured Alerts"),
            alerts.length === 0 ? (React.createElement("div", { className: "empty-state" },
                React.createElement("h4", null, "No alerts configured"),
                React.createElement("p", null, "Create your first alert to get notified of security events"),
                React.createElement("button", { className: "btn btn-primary", onClick: () => setShowCreateModal(true) }, "Create Alert"))) : (React.createElement("div", { className: "alerts-list" }, alerts.map((alert) => (React.createElement(AlertCard, { key: alert.id, alert: alert, onToggle: () => handleToggleAlert(alert), onEdit: () => handleEditAlert(alert), onDelete: () => handleDeleteAlert(alert.id) })))))),
        showCreateModal && (React.createElement(AlertModal, { alert: editingAlert, onClose: () => {
                setShowCreateModal(false);
                setEditingAlert(null);
            }, onSave: (savedAlert) => {
                if (editingAlert) {
                    setAlerts((prev) => prev.map((a) => (a.id === savedAlert.id ? savedAlert : a)));
                }
                else {
                    setAlerts((prev) => [savedAlert, ...prev]);
                }
                setShowCreateModal(false);
                setEditingAlert(null);
            } }))));
};
function AlertTemplate({ title, description, icon, onClick, }) {
    return (React.createElement("button", { className: "alert-template", onClick: onClick },
        React.createElement("div", { className: "template-icon" }, icon),
        React.createElement("div", { className: "template-content" },
            React.createElement("div", { className: "template-title" }, title),
            React.createElement("div", { className: "template-description" }, description))));
}
function AlertCard({ alert, onToggle, onEdit, onDelete, }) {
    return (React.createElement("div", { className: `alert-card ${alert.enabled ? 'enabled' : 'disabled'}` },
        React.createElement("div", { className: "alert-header" },
            React.createElement("div", { className: "alert-title-section" },
                React.createElement("h4", null, alert.name),
                React.createElement("p", { className: "alert-description" }, alert.description)),
            React.createElement("div", { className: "alert-toggle" },
                React.createElement("label", { className: "toggle-switch" },
                    React.createElement("input", { type: "checkbox", checked: alert.enabled, onChange: onToggle }),
                    React.createElement("span", { className: "toggle-slider" })))),
        React.createElement("div", { className: "alert-conditions" },
            React.createElement("h5", null, "Trigger Conditions"),
            React.createElement("div", { className: "conditions-grid" },
                alert.conditions.event_types && (React.createElement("div", { className: "condition-item" },
                    React.createElement("label", null, "Event Types:"),
                    React.createElement("div", { className: "condition-values" }, alert.conditions.event_types.map((type) => (React.createElement("span", { key: type, className: "condition-tag" }, type)))))),
                alert.conditions.severities && (React.createElement("div", { className: "condition-item" },
                    React.createElement("label", null, "Severities:"),
                    React.createElement("div", { className: "condition-values" }, alert.conditions.severities.map((severity) => (React.createElement("span", { key: severity, className: `severity-tag severity-${severity}` }, severity.toUpperCase())))))),
                alert.conditions.min_risk_score !== undefined && (React.createElement("div", { className: "condition-item" },
                    React.createElement("label", null, "Minimum Risk Score:"),
                    React.createElement("span", { className: "condition-value" }, alert.conditions.min_risk_score))),
                alert.conditions.threshold && (React.createElement("div", { className: "condition-item" },
                    React.createElement("label", null, "Threshold:"),
                    React.createElement("span", { className: "condition-value" },
                        alert.conditions.threshold.count,
                        " events in",
                        ' ',
                        alert.conditions.threshold.window_seconds,
                        "s"))))),
        React.createElement("div", { className: "alert-notifications" },
            React.createElement("h5", null, "Notifications"),
            React.createElement("div", { className: "notification-channels" }, alert.notification_channels.map((channel) => (React.createElement("span", { key: channel, className: "channel-badge" },
                channel === 'email' && '📧',
                channel === 'slack' && '💬',
                channel === 'webhook' && '🔗',
                channel === 'sms' && '📱',
                channel)))),
            React.createElement("div", { className: "notification-recipients" }, alert.notification_recipients.join(', '))),
        React.createElement("div", { className: "alert-stats" },
            React.createElement("div", { className: "stat-item" },
                React.createElement("label", null, "Cooldown:"),
                React.createElement("span", null,
                    alert.cooldown_minutes,
                    " minutes")),
            React.createElement("div", { className: "stat-item" },
                React.createElement("label", null, "Triggered:"),
                React.createElement("span", null,
                    alert.trigger_count,
                    " times")),
            alert.last_triggered && (React.createElement("div", { className: "stat-item" },
                React.createElement("label", null, "Last Triggered:"),
                React.createElement("span", null, new Date(alert.last_triggered).toLocaleString())))),
        React.createElement("div", { className: "alert-actions" },
            React.createElement("button", { className: "btn btn-sm btn-secondary", onClick: onEdit }, "Edit"),
            React.createElement("button", { className: "btn btn-sm btn-danger", onClick: onDelete }, "Delete"))));
}
function AlertModal({ alert, onClose, onSave, }) {
    const [name, setName] = useState(alert?.name || '');
    const [description, setDescription] = useState(alert?.description || '');
    const [eventTypes, setEventTypes] = useState(alert?.conditions.event_types || []);
    const [severities, setSeverities] = useState(alert?.conditions.severities || []);
    const [minRiskScore, setMinRiskScore] = useState(alert?.conditions.min_risk_score || 70);
    const [thresholdCount, setThresholdCount] = useState(alert?.conditions.threshold?.count || 5);
    const [thresholdWindow, setThresholdWindow] = useState(alert?.conditions.threshold?.window_seconds || 300);
    const [channels, setChannels] = useState(alert?.notification_channels || ['email']);
    const [recipients, setRecipients] = useState(alert?.notification_recipients.join(', ') || '');
    const [cooldown, setCooldown] = useState(alert?.cooldown_minutes || 60);
    const [errors, setErrors] = useState({});
    const validate = () => {
        const newErrors = {};
        if (!name.trim()) {
            newErrors.name = 'Alert name is required';
        }
        if (!recipients.trim()) {
            newErrors.recipients = 'At least one recipient is required';
        }
        setErrors(newErrors);
        return Object.keys(newErrors).length === 0;
    };
    const handleSave = async () => {
        if (!validate())
            return;
        const alertData = {
            id: alert?.id || crypto.randomUUID(),
            name,
            description,
            enabled: alert?.enabled ?? true,
            conditions: {
                event_types: eventTypes.length > 0 ? eventTypes : undefined,
                severities: severities.length > 0 ? severities : undefined,
                min_risk_score: minRiskScore,
                threshold: {
                    count: thresholdCount,
                    window_seconds: thresholdWindow,
                },
            },
            notification_channels: channels,
            notification_recipients: recipients.split(',').map((r) => r.trim()),
            cooldown_minutes: cooldown,
            created_by: alert?.created_by || 'current_user',
            created_at: alert?.created_at || new Date().toISOString(),
            updated_at: new Date().toISOString(),
            last_triggered: alert?.last_triggered,
            trigger_count: alert?.trigger_count || 0,
        };
        try {
            const response = await fetch(alert ? `/api/audit/alerts/${alert.id}` : '/api/audit/alerts', {
                method: alert ? 'PUT' : 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(alertData),
            });
            if (response.ok) {
                const savedAlert = await response.json();
                onSave(savedAlert);
            }
        }
        catch (error) {
            console.error('Failed to save alert:', error);
        }
    };
    const toggleSeverity = (severity) => {
        setSeverities((prev) => prev.includes(severity)
            ? prev.filter((s) => s !== severity)
            : [...prev, severity]);
    };
    const toggleChannel = (channel) => {
        setChannels((prev) => prev.includes(channel) ? prev.filter((c) => c !== channel) : [...prev, channel]);
    };
    return (React.createElement("div", { className: "modal-overlay", onClick: onClose },
        React.createElement("div", { className: "modal alert-modal", onClick: (e) => e.stopPropagation() },
            React.createElement("div", { className: "modal-header" },
                React.createElement("h2", null, alert ? 'Edit Alert' : 'Create Alert'),
                React.createElement("button", { className: "modal-close", onClick: onClose }, "\u00D7")),
            React.createElement("div", { className: "modal-content" },
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Alert Name *"),
                    React.createElement("input", { type: "text", value: name, onChange: (e) => setName(e.target.value), placeholder: "e.g., Suspicious Login Activity", className: errors.name ? 'error' : '' }),
                    errors.name && React.createElement("span", { className: "error-message" }, errors.name)),
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Description"),
                    React.createElement("textarea", { value: description, onChange: (e) => setDescription(e.target.value), placeholder: "Describe what this alert monitors", rows: 3 })),
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Severity Levels"),
                    React.createElement("div", { className: "severity-filters" }, ['low', 'medium', 'high', 'critical'].map((severity) => (React.createElement("button", { key: severity, className: `severity-chip severity-${severity} ${severities.includes(severity) ? 'active' : ''}`, onClick: () => toggleSeverity(severity) }, severity.toUpperCase()))))),
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null,
                        "Minimum Risk Score: ",
                        minRiskScore),
                    React.createElement("input", { type: "range", min: "0", max: "100", step: "10", value: minRiskScore, onChange: (e) => setMinRiskScore(parseInt(e.target.value)), className: "risk-slider" })),
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Threshold"),
                    React.createElement("div", { className: "form-row" },
                        React.createElement("input", { type: "number", value: thresholdCount, onChange: (e) => setThresholdCount(parseInt(e.target.value)), placeholder: "Count", min: "1" }),
                        React.createElement("span", null, "events in"),
                        React.createElement("input", { type: "number", value: thresholdWindow, onChange: (e) => setThresholdWindow(parseInt(e.target.value)), placeholder: "Seconds", min: "1" }),
                        React.createElement("span", null, "seconds"))),
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Notification Channels"),
                    React.createElement("div", { className: "channel-options" }, ['email', 'slack', 'webhook', 'sms'].map((channel) => (React.createElement("button", { key: channel, className: `channel-chip ${channels.includes(channel) ? 'active' : ''}`, onClick: () => toggleChannel(channel) },
                        channel === 'email' && '📧 ',
                        channel === 'slack' && '💬 ',
                        channel === 'webhook' && '🔗 ',
                        channel === 'sms' && '📱 ',
                        channel))))),
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Recipients (comma-separated) *"),
                    React.createElement("input", { type: "text", value: recipients, onChange: (e) => setRecipients(e.target.value), placeholder: "user@example.com, admin@example.com", className: errors.recipients ? 'error' : '' }),
                    errors.recipients && (React.createElement("span", { className: "error-message" }, errors.recipients))),
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Cooldown Period (minutes)"),
                    React.createElement("input", { type: "number", value: cooldown, onChange: (e) => setCooldown(parseInt(e.target.value)), min: "1", placeholder: "60" }),
                    React.createElement("small", null, "Minimum time between alert notifications"))),
            React.createElement("div", { className: "modal-footer" },
                React.createElement("button", { className: "btn btn-secondary", onClick: onClose }, "Cancel"),
                React.createElement("button", { className: "btn btn-primary", onClick: handleSave }, alert ? 'Save Changes' : 'Create Alert')))));
}
//# sourceMappingURL=AuditAlerts.js.map