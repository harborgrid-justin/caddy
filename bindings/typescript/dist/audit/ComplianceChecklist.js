import React, { useState, useEffect } from 'react';
export const ComplianceChecklist = ({ organizationId, framework: initialFramework, }) => {
    const [selectedFramework, setSelectedFramework] = useState(initialFramework || 'SOC2');
    const [requirements, setRequirements] = useState([]);
    const [loading, setLoading] = useState(true);
    const [filterStatus, setFilterStatus] = useState('all');
    const [searchQuery, setSearchQuery] = useState('');
    const [selectedCategory, setSelectedCategory] = useState('all');
    useEffect(() => {
        loadRequirements();
    }, [selectedFramework, organizationId]);
    const loadRequirements = async () => {
        setLoading(true);
        try {
            const params = new URLSearchParams({
                framework: selectedFramework,
                ...(organizationId && { organization_id: organizationId }),
            });
            const response = await fetch(`/api/compliance/requirements?${params}`);
            const data = await response.json();
            setRequirements(data.requirements || []);
        }
        catch (error) {
            console.error('Failed to load compliance requirements:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const updateRequirementStatus = async (requirementId, status) => {
        try {
            await fetch(`/api/compliance/requirements/${requirementId}/status`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ status }),
            });
            setRequirements((prev) => prev.map((req) => req.id === requirementId ? { ...req, status } : req));
        }
        catch (error) {
            console.error('Failed to update requirement status:', error);
        }
    };
    const filteredRequirements = requirements.filter((req) => {
        if (filterStatus !== 'all' && req.status !== filterStatus)
            return false;
        if (selectedCategory !== 'all' && req.category !== selectedCategory)
            return false;
        if (searchQuery && !req.title.toLowerCase().includes(searchQuery.toLowerCase()) &&
            !req.description.toLowerCase().includes(searchQuery.toLowerCase())) {
            return false;
        }
        return true;
    });
    const categories = Array.from(new Set(requirements.map((r) => r.category)));
    const complianceStats = {
        total: requirements.length,
        compliant: requirements.filter((r) => r.status === 'compliant').length,
        non_compliant: requirements.filter((r) => r.status === 'non_compliant').length,
        partial: requirements.filter((r) => r.status === 'partial').length,
        not_applicable: requirements.filter((r) => r.status === 'not_applicable').length,
    };
    const overallCompliance = requirements.length > 0
        ? ((complianceStats.compliant + complianceStats.partial * 0.5) / requirements.length) * 100
        : 0;
    return (React.createElement("div", { className: "compliance-checklist" },
        React.createElement("div", { className: "checklist-header" },
            React.createElement("div", null,
                React.createElement("h2", null, "Compliance Checklist"),
                React.createElement("p", { className: "subtitle" }, "Track and manage compliance requirements across frameworks"))),
        React.createElement("div", { className: "framework-selector" }, ['SOC2', 'GDPR', 'HIPAA', 'ISO27001', 'PCI_DSS', 'CCPA', 'NIST', 'FedRAMP'].map((framework) => (React.createElement("button", { key: framework, className: `framework-tab ${selectedFramework === framework ? 'active' : ''}`, onClick: () => setSelectedFramework(framework) }, framework)))),
        React.createElement("div", { className: "compliance-overview" },
            React.createElement("div", { className: "overview-stats" },
                React.createElement(StatCard, { label: "Overall Compliance", value: `${overallCompliance.toFixed(1)}%`, color: overallCompliance >= 80 ? 'green' : overallCompliance >= 60 ? 'yellow' : 'red' }),
                React.createElement(StatCard, { label: "Compliant", value: complianceStats.compliant.toString(), color: "green" }),
                React.createElement(StatCard, { label: "Partial", value: complianceStats.partial.toString(), color: "yellow" }),
                React.createElement(StatCard, { label: "Non-Compliant", value: complianceStats.non_compliant.toString(), color: "red" })),
            React.createElement("div", { className: "progress-bar-container" },
                React.createElement("div", { className: "progress-bar" },
                    React.createElement("div", { className: "progress-segment compliant", style: {
                            width: `${(complianceStats.compliant / requirements.length) * 100}%`,
                        } }),
                    React.createElement("div", { className: "progress-segment partial", style: {
                            width: `${(complianceStats.partial / requirements.length) * 100}%`,
                        } }),
                    React.createElement("div", { className: "progress-segment non-compliant", style: {
                            width: `${(complianceStats.non_compliant / requirements.length) * 100}%`,
                        } })))),
        React.createElement("div", { className: "checklist-filters" },
            React.createElement("input", { type: "text", className: "search-input", placeholder: "Search requirements...", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value) }),
            React.createElement("select", { value: selectedCategory, onChange: (e) => setSelectedCategory(e.target.value), className: "category-select" },
                React.createElement("option", { value: "all" }, "All Categories"),
                categories.map((category) => (React.createElement("option", { key: category, value: category }, category)))),
            React.createElement("div", { className: "status-filters" }, ['all', 'compliant', 'partial', 'non_compliant'].map((status) => (React.createElement("button", { key: status, className: `status-filter ${filterStatus === status ? 'active' : ''}`, onClick: () => setFilterStatus(status) }, status === 'all' ? 'All' : status.replace('_', ' ')))))),
        loading ? (React.createElement("div", { className: "loading-state" },
            React.createElement("div", { className: "loading-spinner" }),
            React.createElement("p", null, "Loading compliance requirements..."))) : (React.createElement("div", { className: "requirements-list" }, filteredRequirements.length === 0 ? (React.createElement("div", { className: "empty-state" },
            React.createElement("p", null, "No requirements match your filters"))) : (filteredRequirements.map((requirement) => (React.createElement(RequirementCard, { key: requirement.id, requirement: requirement, onUpdateStatus: updateRequirementStatus }))))))));
};
function StatCard({ label, value, color, }) {
    return (React.createElement("div", { className: `stat-card stat-${color}` },
        React.createElement("div", { className: "stat-value" }, value),
        React.createElement("div", { className: "stat-label" }, label)));
}
function RequirementCard({ requirement, onUpdateStatus, }) {
    const [expanded, setExpanded] = useState(false);
    const [showEvidenceModal, setShowEvidenceModal] = useState(false);
    const statusColors = {
        compliant: 'green',
        partial: 'yellow',
        non_compliant: 'red',
        not_applicable: 'gray',
    };
    const statusIcons = {
        compliant: '✓',
        partial: '◐',
        non_compliant: '✗',
        not_applicable: '○',
    };
    return (React.createElement("div", { className: `requirement-card status-${requirement.status}` },
        React.createElement("div", { className: "requirement-header", onClick: () => setExpanded(!expanded) },
            React.createElement("div", { className: "requirement-title-section" },
                React.createElement("div", { className: "requirement-id" }, requirement.requirement_id),
                React.createElement("div", { className: "requirement-title" }, requirement.title),
                React.createElement("div", { className: "requirement-category" }, requirement.category)),
            React.createElement("div", { className: "requirement-status-section" },
                React.createElement("div", { className: "compliance-percentage" },
                    requirement.compliance_percentage.toFixed(0),
                    "%"),
                React.createElement("div", { className: `status-badge badge-${statusColors[requirement.status]}` },
                    statusIcons[requirement.status],
                    " ",
                    requirement.status.replace('_', ' ')),
                React.createElement("button", { className: "expand-button" }, expanded ? '▼' : '▶'))),
        expanded && (React.createElement("div", { className: "requirement-details" },
            React.createElement("div", { className: "requirement-description" }, requirement.description),
            React.createElement("div", { className: "evidence-section" },
                React.createElement("div", { className: "evidence-header" },
                    React.createElement("h4", null,
                        "Evidence (",
                        requirement.evidence_collected.length,
                        "/",
                        requirement.evidence_required.length,
                        ")"),
                    React.createElement("button", { className: "btn btn-sm btn-secondary", onClick: () => setShowEvidenceModal(true) }, "Add Evidence")),
                requirement.evidence_required.length > 0 && (React.createElement("div", { className: "evidence-required" },
                    React.createElement("strong", null, "Required:"),
                    React.createElement("ul", null, requirement.evidence_required.map((evidence, index) => (React.createElement("li", { key: index }, evidence)))))),
                requirement.evidence_collected.length > 0 && (React.createElement("div", { className: "evidence-collected" },
                    React.createElement("strong", null, "Collected:"),
                    React.createElement("ul", null, requirement.evidence_collected.map((evidence, index) => (React.createElement("li", { key: index },
                        React.createElement("div", { className: "evidence-type" }, evidence.type),
                        React.createElement("div", { className: "evidence-description" }, evidence.description),
                        React.createElement("div", { className: "evidence-meta" },
                            "Collected by ",
                            evidence.collected_by,
                            " on",
                            ' ',
                            new Date(evidence.collected_at).toLocaleDateString())))))))),
            requirement.remediation_required && requirement.remediation_tasks && (React.createElement("div", { className: "remediation-section" },
                React.createElement("h4", null, "Remediation Tasks"),
                React.createElement("ul", { className: "remediation-tasks" }, requirement.remediation_tasks.map((task) => (React.createElement("li", { key: task.id, className: `task-${task.status}` },
                    React.createElement("div", { className: "task-description" }, task.description),
                    React.createElement("div", { className: "task-meta" },
                        task.assigned_to && React.createElement("span", null,
                            "Assigned to: ",
                            task.assigned_to),
                        task.due_date && (React.createElement("span", null,
                            "Due: ",
                            new Date(task.due_date).toLocaleDateString())),
                        React.createElement("span", { className: `status-badge badge-${task.status}` }, task.status)))))))),
            React.createElement("div", { className: "assessment-info" },
                React.createElement("div", { className: "info-item" },
                    React.createElement("strong", null, "Last Assessed:"),
                    ' ',
                    new Date(requirement.last_assessed).toLocaleDateString()),
                React.createElement("div", { className: "info-item" },
                    React.createElement("strong", null, "Assessed By:"),
                    " ",
                    requirement.assessed_by),
                React.createElement("div", { className: "info-item" },
                    React.createElement("strong", null, "Next Assessment:"),
                    ' ',
                    new Date(requirement.next_assessment).toLocaleDateString())),
            React.createElement("div", { className: "status-update-section" },
                React.createElement("label", null, "Update Status:"),
                React.createElement("div", { className: "status-buttons" }, ['compliant', 'partial', 'non_compliant', 'not_applicable'].map((status) => (React.createElement("button", { key: status, className: `status-button ${requirement.status === status ? 'active' : ''} status-${statusColors[status]}`, onClick: () => onUpdateStatus(requirement.id, status) },
                    statusIcons[status],
                    " ",
                    status.replace('_', ' '))))))))));
}
//# sourceMappingURL=ComplianceChecklist.js.map