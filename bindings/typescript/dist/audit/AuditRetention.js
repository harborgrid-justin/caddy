import React, { useState, useEffect } from 'react';
export const AuditRetention = ({ organizationId }) => {
    const [rules, setRules] = useState([]);
    const [loading, setLoading] = useState(true);
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [editingRule, setEditingRule] = useState(null);
    useEffect(() => {
        loadRetentionRules();
    }, [organizationId]);
    const loadRetentionRules = async () => {
        setLoading(true);
        try {
            const params = new URLSearchParams(organizationId ? { organization_id: organizationId } : {});
            const response = await fetch(`/api/audit/retention?${params}`);
            const data = await response.json();
            setRules(data.rules || []);
        }
        catch (error) {
            console.error('Failed to load retention rules:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const handleCreateRule = () => {
        setEditingRule(null);
        setShowCreateModal(true);
    };
    const handleEditRule = (rule) => {
        setEditingRule(rule);
        setShowCreateModal(true);
    };
    const handleDeleteRule = async (ruleId) => {
        if (!confirm('Are you sure you want to delete this retention rule?'))
            return;
        try {
            await fetch(`/api/audit/retention/${ruleId}`, {
                method: 'DELETE',
            });
            setRules((prev) => prev.filter((r) => r.id !== ruleId));
        }
        catch (error) {
            console.error('Failed to delete retention rule:', error);
        }
    };
    const handleToggleRule = async (rule) => {
        try {
            const response = await fetch(`/api/audit/retention/${rule.id}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ enabled: !rule.enabled }),
            });
            const updated = await response.json();
            setRules((prev) => prev.map((r) => (r.id === rule.id ? updated : r)));
        }
        catch (error) {
            console.error('Failed to toggle retention rule:', error);
        }
    };
    if (loading) {
        return (React.createElement("div", { className: "audit-retention loading" },
            React.createElement("div", { className: "loading-spinner" }),
            React.createElement("p", null, "Loading retention rules...")));
    }
    return (React.createElement("div", { className: "audit-retention" },
        React.createElement("div", { className: "retention-header" },
            React.createElement("div", null,
                React.createElement("h2", null, "Audit Log Retention"),
                React.createElement("p", { className: "subtitle" }, "Manage data retention policies for compliance and storage optimization")),
            React.createElement("button", { className: "btn btn-primary", onClick: handleCreateRule }, "Create Retention Rule")),
        React.createElement("div", { className: "retention-overview" },
            React.createElement(OverviewCard, { title: "Total Rules", value: rules.length.toString(), icon: "\uD83D\uDCCB" }),
            React.createElement(OverviewCard, { title: "Active Rules", value: rules.filter((r) => r.enabled).length.toString(), icon: "\u2713" }),
            React.createElement(OverviewCard, { title: "Legal Holds", value: rules.filter((r) => r.legal_hold).length.toString(), icon: "\u2696\uFE0F" })),
        React.createElement("div", { className: "retention-rules" }, rules.length === 0 ? (React.createElement("div", { className: "empty-state" },
            React.createElement("h3", null, "No Retention Rules"),
            React.createElement("p", null, "Create your first retention rule to manage audit log lifecycle."),
            React.createElement("button", { className: "btn btn-primary", onClick: handleCreateRule }, "Create Retention Rule"))) : (React.createElement("table", { className: "rules-table" },
            React.createElement("thead", null,
                React.createElement("tr", null,
                    React.createElement("th", null, "Rule Name"),
                    React.createElement("th", null, "Scope"),
                    React.createElement("th", null, "Retention Period"),
                    React.createElement("th", null, "Archive"),
                    React.createElement("th", null, "Delete"),
                    React.createElement("th", null, "Status"),
                    React.createElement("th", null, "Actions"))),
            React.createElement("tbody", null, rules.map((rule) => (React.createElement("tr", { key: rule.id, className: !rule.enabled ? 'disabled' : '' },
                React.createElement("td", null,
                    React.createElement("div", { className: "rule-name" },
                        rule.name,
                        rule.legal_hold && (React.createElement("span", { className: "badge badge-warning", title: "Legal Hold" }, "\u2696\uFE0F"))),
                    React.createElement("div", { className: "rule-description" }, rule.description)),
                React.createElement("td", null,
                    React.createElement(RuleScope, { rule: rule })),
                React.createElement("td", null,
                    rule.retention_days,
                    " days"),
                React.createElement("td", null, rule.archive_after_days ? `${rule.archive_after_days} days` : '-'),
                React.createElement("td", null, rule.delete_after_days ? `${rule.delete_after_days} days` : 'Never'),
                React.createElement("td", null,
                    React.createElement("label", { className: "toggle-switch" },
                        React.createElement("input", { type: "checkbox", checked: rule.enabled, onChange: () => handleToggleRule(rule) }),
                        React.createElement("span", { className: "toggle-slider" }))),
                React.createElement("td", { className: "actions-cell" },
                    React.createElement("button", { className: "btn-icon", onClick: () => handleEditRule(rule), title: "Edit" }, "\u270F\uFE0F"),
                    React.createElement("button", { className: "btn-icon", onClick: () => handleDeleteRule(rule.id), title: "Delete", disabled: rule.legal_hold }, "\uD83D\uDDD1\uFE0F"))))))))),
        React.createElement("div", { className: "best-practices" },
            React.createElement("h3", null, "Retention Best Practices"),
            React.createElement("div", { className: "practices-grid" },
                React.createElement("div", { className: "practice-card" },
                    React.createElement("h4", null, "SOC 2 Compliance"),
                    React.createElement("p", null, "Retain audit logs for at least 1 year, with 90-day retention for active logs."),
                    React.createElement("span", { className: "recommended" }, "Recommended: 365 days")),
                React.createElement("div", { className: "practice-card" },
                    React.createElement("h4", null, "GDPR Compliance"),
                    React.createElement("p", null, "Delete personal data when no longer necessary, typically within 2 years."),
                    React.createElement("span", { className: "recommended" }, "Recommended: 730 days")),
                React.createElement("div", { className: "practice-card" },
                    React.createElement("h4", null, "HIPAA Compliance"),
                    React.createElement("p", null, "Retain audit logs for at least 6 years from date of creation or last use."),
                    React.createElement("span", { className: "recommended" }, "Recommended: 2190 days")),
                React.createElement("div", { className: "practice-card" },
                    React.createElement("h4", null, "PCI DSS Compliance"),
                    React.createElement("p", null, "Retain audit trail history for at least one year with 90 days immediately available."),
                    React.createElement("span", { className: "recommended" }, "Recommended: 365 days")))),
        showCreateModal && (React.createElement(RetentionRuleModal, { rule: editingRule, onSave: (savedRule) => {
                if (editingRule) {
                    setRules((prev) => prev.map((r) => (r.id === savedRule.id ? savedRule : r)));
                }
                else {
                    setRules((prev) => [savedRule, ...prev]);
                }
                setShowCreateModal(false);
                setEditingRule(null);
            }, onClose: () => {
                setShowCreateModal(false);
                setEditingRule(null);
            } }))));
};
function OverviewCard({ title, value, icon, }) {
    return (React.createElement("div", { className: "overview-card" },
        React.createElement("div", { className: "card-icon" }, icon),
        React.createElement("div", { className: "card-content" },
            React.createElement("div", { className: "card-value" }, value),
            React.createElement("div", { className: "card-title" }, title))));
}
function RuleScope({ rule }) {
    const scopes = [];
    if (rule.event_types && rule.event_types.length > 0) {
        scopes.push(`${rule.event_types.length} event types`);
    }
    if (rule.severities && rule.severities.length > 0) {
        scopes.push(`${rule.severities.length} severities`);
    }
    if (rule.data_classifications && rule.data_classifications.length > 0) {
        scopes.push(`${rule.data_classifications.length} classifications`);
    }
    if (rule.compliance_frameworks && rule.compliance_frameworks.length > 0) {
        scopes.push(`${rule.compliance_frameworks.length} frameworks`);
    }
    return (React.createElement("div", { className: "rule-scope" }, scopes.length > 0 ? scopes.join(', ') : 'All events'));
}
function RetentionRuleModal({ rule, onSave, onClose, }) {
    const [name, setName] = useState(rule?.name || '');
    const [description, setDescription] = useState(rule?.description || '');
    const [retentionDays, setRetentionDays] = useState(rule?.retention_days || 365);
    const [archiveDays, setArchiveDays] = useState(rule?.archive_after_days || undefined);
    const [deleteDays, setDeleteDays] = useState(rule?.delete_after_days || undefined);
    const [legalHold, setLegalHold] = useState(rule?.legal_hold || false);
    const [legalHoldReason, setLegalHoldReason] = useState(rule?.legal_hold_reason || '');
    const [selectedFrameworks, setSelectedFrameworks] = useState(rule?.compliance_frameworks || []);
    const [errors, setErrors] = useState({});
    const validate = () => {
        const newErrors = {};
        if (!name.trim()) {
            newErrors.name = 'Rule name is required';
        }
        if (retentionDays < 1) {
            newErrors.retentionDays = 'Retention period must be at least 1 day';
        }
        if (archiveDays && archiveDays >= retentionDays) {
            newErrors.archiveDays = 'Archive period must be less than retention period';
        }
        if (deleteDays && deleteDays <= retentionDays) {
            newErrors.deleteDays = 'Delete period must be greater than retention period';
        }
        if (legalHold && !legalHoldReason.trim()) {
            newErrors.legalHoldReason = 'Legal hold reason is required';
        }
        setErrors(newErrors);
        return Object.keys(newErrors).length === 0;
    };
    const handleSave = async () => {
        if (!validate())
            return;
        const ruleData = {
            id: rule?.id || crypto.randomUUID(),
            name,
            description,
            enabled: rule?.enabled ?? true,
            retention_days: retentionDays,
            archive_after_days: archiveDays,
            delete_after_days: deleteDays,
            legal_hold: legalHold,
            legal_hold_reason: legalHold ? legalHoldReason : undefined,
            compliance_frameworks: selectedFrameworks.length > 0 ? selectedFrameworks : undefined,
            created_by: rule?.created_by || 'current_user',
            created_at: rule?.created_at || new Date().toISOString(),
            updated_at: new Date().toISOString(),
        };
        try {
            const response = await fetch(rule ? `/api/audit/retention/${rule.id}` : '/api/audit/retention', {
                method: rule ? 'PUT' : 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(ruleData),
            });
            if (response.ok) {
                const savedRule = await response.json();
                onSave(savedRule);
            }
        }
        catch (error) {
            console.error('Failed to save retention rule:', error);
        }
    };
    const toggleFramework = (framework) => {
        setSelectedFrameworks((prev) => prev.includes(framework)
            ? prev.filter((f) => f !== framework)
            : [...prev, framework]);
    };
    return (React.createElement("div", { className: "modal-overlay", onClick: onClose },
        React.createElement("div", { className: "modal retention-modal", onClick: (e) => e.stopPropagation() },
            React.createElement("div", { className: "modal-header" },
                React.createElement("h2", null, rule ? 'Edit Retention Rule' : 'Create Retention Rule'),
                React.createElement("button", { className: "modal-close", onClick: onClose }, "\u00D7")),
            React.createElement("div", { className: "modal-content" },
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Rule Name *"),
                    React.createElement("input", { type: "text", value: name, onChange: (e) => setName(e.target.value), placeholder: "e.g., Standard Retention Policy", className: errors.name ? 'error' : '' }),
                    errors.name && React.createElement("span", { className: "error-message" }, errors.name)),
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Description"),
                    React.createElement("textarea", { value: description, onChange: (e) => setDescription(e.target.value), placeholder: "Describe the purpose of this retention rule", rows: 3 })),
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Retention Period (days) *"),
                    React.createElement("input", { type: "number", value: retentionDays, onChange: (e) => setRetentionDays(parseInt(e.target.value) || 0), min: "1", className: errors.retentionDays ? 'error' : '' }),
                    errors.retentionDays && (React.createElement("span", { className: "error-message" }, errors.retentionDays))),
                React.createElement("div", { className: "form-row" },
                    React.createElement("div", { className: "form-section" },
                        React.createElement("label", null, "Archive After (days)"),
                        React.createElement("input", { type: "number", value: archiveDays || '', onChange: (e) => setArchiveDays(e.target.value ? parseInt(e.target.value) : undefined), placeholder: "Optional", min: "1", className: errors.archiveDays ? 'error' : '' }),
                        errors.archiveDays && (React.createElement("span", { className: "error-message" }, errors.archiveDays))),
                    React.createElement("div", { className: "form-section" },
                        React.createElement("label", null, "Delete After (days)"),
                        React.createElement("input", { type: "number", value: deleteDays || '', onChange: (e) => setDeleteDays(e.target.value ? parseInt(e.target.value) : undefined), placeholder: "Never", min: "1", className: errors.deleteDays ? 'error' : '' }),
                        errors.deleteDays && (React.createElement("span", { className: "error-message" }, errors.deleteDays)))),
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", null, "Compliance Frameworks"),
                    React.createElement("div", { className: "framework-chips" }, ['SOC2', 'GDPR', 'HIPAA', 'ISO27001', 'PCI_DSS', 'CCPA', 'NIST', 'FedRAMP'].map((framework) => (React.createElement("button", { key: framework, className: `chip ${selectedFrameworks.includes(framework) ? 'active' : ''}`, onClick: () => toggleFramework(framework) }, framework))))),
                React.createElement("div", { className: "form-section" },
                    React.createElement("label", { className: "checkbox-label" },
                        React.createElement("input", { type: "checkbox", checked: legalHold, onChange: (e) => setLegalHold(e.target.checked) }),
                        React.createElement("span", null, "Legal Hold")),
                    legalHold && (React.createElement("div", { className: "legal-hold-reason" },
                        React.createElement("input", { type: "text", value: legalHoldReason, onChange: (e) => setLegalHoldReason(e.target.value), placeholder: "Enter legal hold reason", className: errors.legalHoldReason ? 'error' : '' }),
                        errors.legalHoldReason && (React.createElement("span", { className: "error-message" }, errors.legalHoldReason)))))),
            React.createElement("div", { className: "modal-footer" },
                React.createElement("button", { className: "btn btn-secondary", onClick: onClose }, "Cancel"),
                React.createElement("button", { className: "btn btn-primary", onClick: handleSave }, rule ? 'Save Changes' : 'Create Rule')))));
}
//# sourceMappingURL=AuditRetention.js.map