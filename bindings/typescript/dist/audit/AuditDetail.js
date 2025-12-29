import React, { useState, useEffect } from 'react';
export const AuditDetail = ({ eventId, event: providedEvent, onClose, }) => {
    const [event, setEvent] = useState(providedEvent || null);
    const [loading, setLoading] = useState(!providedEvent);
    const [verifying, setVerifying] = useState(false);
    const [integrityStatus, setIntegrityStatus] = useState(null);
    const [anomalyDetails, setAnomalyDetails] = useState(null);
    const [activeTab, setActiveTab] = useState('overview');
    useEffect(() => {
        if (!providedEvent) {
            loadEventDetails();
        }
    }, [eventId, providedEvent]);
    const loadEventDetails = async () => {
        setLoading(true);
        try {
            const response = await fetch(`/api/audit/events/${eventId}`);
            const data = await response.json();
            setEvent(data);
            if (data.anomaly_detected) {
                const anomalyResponse = await fetch(`/api/audit/events/${eventId}/anomaly`);
                const anomalyData = await anomalyResponse.json();
                setAnomalyDetails(anomalyData);
            }
        }
        catch (error) {
            console.error('Failed to load event details:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const verifyIntegrity = async () => {
        if (!event)
            return;
        setVerifying(true);
        try {
            const response = await fetch(`/api/audit/events/${event.id}/verify`, {
                method: 'POST',
            });
            const data = await response.json();
            setIntegrityStatus(data.verified ? 'verified' : 'failed');
        }
        catch (error) {
            console.error('Failed to verify integrity:', error);
            setIntegrityStatus('failed');
        }
        finally {
            setVerifying(false);
        }
    };
    if (loading) {
        return (React.createElement("div", { className: "audit-detail loading" },
            React.createElement("div", { className: "loading-spinner" }),
            React.createElement("p", null, "Loading event details...")));
    }
    if (!event) {
        return (React.createElement("div", { className: "audit-detail error" },
            React.createElement("h2", null, "Event Not Found"),
            React.createElement("p", null, "The requested audit event could not be found."),
            onClose && (React.createElement("button", { onClick: onClose, className: "btn btn-secondary" }, "Close"))));
    }
    return (React.createElement("div", { className: "audit-detail" },
        React.createElement("div", { className: "detail-header" },
            React.createElement("div", { className: "header-content" },
                React.createElement("h2", null, "Audit Event Details"),
                React.createElement("div", { className: "event-badges" },
                    React.createElement(SeverityBadge, { severity: event.severity }),
                    React.createElement(StatusBadge, { status: event.status }),
                    event.anomaly_detected && (React.createElement("span", { className: "badge badge-warning" }, "Anomaly Detected")),
                    event.risk_score !== undefined && event.risk_score >= 70 && (React.createElement("span", { className: "badge badge-danger" }, "High Risk")))),
            React.createElement("div", { className: "header-actions" },
                React.createElement("button", { className: "btn btn-secondary", onClick: verifyIntegrity, disabled: verifying }, verifying ? 'Verifying...' : 'Verify Integrity'),
                onClose && (React.createElement("button", { onClick: onClose, className: "btn btn-ghost" }, "Close")))),
        integrityStatus && (React.createElement("div", { className: `integrity-status status-${integrityStatus}` }, integrityStatus === 'verified' ? (React.createElement(React.Fragment, null,
            React.createElement("span", { className: "status-icon" }, "\u2713"),
            React.createElement("span", null, "Event integrity verified - hash chain intact"))) : (React.createElement(React.Fragment, null,
            React.createElement("span", { className: "status-icon" }, "\u2717"),
            React.createElement("span", null, "Integrity verification failed - possible tampering detected"))))),
        React.createElement("div", { className: "detail-tabs" },
            React.createElement("button", { className: `tab ${activeTab === 'overview' ? 'active' : ''}`, onClick: () => setActiveTab('overview') }, "Overview"),
            React.createElement("button", { className: `tab ${activeTab === 'metadata' ? 'active' : ''}`, onClick: () => setActiveTab('metadata') }, "Metadata"),
            event.changes && event.changes.length > 0 && (React.createElement("button", { className: `tab ${activeTab === 'changes' ? 'active' : ''}`, onClick: () => setActiveTab('changes') },
                "Changes (",
                event.changes.length,
                ")")),
            React.createElement("button", { className: `tab ${activeTab === 'security' ? 'active' : ''}`, onClick: () => setActiveTab('security') }, "Security")),
        React.createElement("div", { className: "detail-content" },
            activeTab === 'overview' && (React.createElement(OverviewTab, { event: event, anomalyDetails: anomalyDetails })),
            activeTab === 'metadata' && React.createElement(MetadataTab, { event: event }),
            activeTab === 'changes' && event.changes && (React.createElement(ChangesTab, { changes: event.changes })),
            activeTab === 'security' && (React.createElement(SecurityTab, { event: event, anomalyDetails: anomalyDetails })))));
};
function OverviewTab({ event, anomalyDetails, }) {
    return (React.createElement("div", { className: "overview-tab" },
        React.createElement("section", { className: "detail-section" },
            React.createElement("h3", null, "Event Information"),
            React.createElement("div", { className: "detail-grid" },
                React.createElement(DetailField, { label: "Event ID", value: event.id, copyable: true }),
                React.createElement(DetailField, { label: "Timestamp", value: formatTimestamp(event.timestamp) }),
                React.createElement(DetailField, { label: "Event Type", value: formatEventType(event.event_type) }),
                React.createElement(DetailField, { label: "Action", value: event.action }),
                React.createElement(DetailField, { label: "Description", value: event.description, fullWidth: true }))),
        React.createElement("section", { className: "detail-section" },
            React.createElement("h3", null, "Actor Information"),
            React.createElement("div", { className: "detail-grid" },
                React.createElement(DetailField, { label: "User ID", value: event.user_id || 'System', copyable: true }),
                React.createElement(DetailField, { label: "Email", value: event.user_email || 'N/A' }),
                React.createElement(DetailField, { label: "Name", value: event.user_name || 'N/A' }),
                React.createElement(DetailField, { label: "IP Address", value: event.user_ip_address, copyable: true }),
                React.createElement(DetailField, { label: "Session ID", value: event.session_id, copyable: true }),
                React.createElement(DetailField, { label: "User Agent", value: event.user_agent, fullWidth: true, truncate: true })),
            event.location && (React.createElement("div", { className: "location-info" },
                React.createElement("strong", null, "Location:"),
                event.location.city && ` ${event.location.city},`,
                event.location.region && ` ${event.location.region},`,
                event.location.country && ` ${event.location.country}`))),
        event.resource_type && (React.createElement("section", { className: "detail-section" },
            React.createElement("h3", null, "Resource Information"),
            React.createElement("div", { className: "detail-grid" },
                React.createElement(DetailField, { label: "Resource Type", value: event.resource_type }),
                React.createElement(DetailField, { label: "Resource ID", value: event.resource_id || 'N/A', copyable: true }),
                React.createElement(DetailField, { label: "Resource Name", value: event.resource_name || 'N/A', fullWidth: true })))),
        anomalyDetails && (React.createElement("section", { className: "detail-section anomaly-section" },
            React.createElement("h3", null, "Anomaly Detection"),
            React.createElement("div", { className: "anomaly-info" },
                React.createElement("div", { className: "anomaly-header" },
                    React.createElement("span", { className: "anomaly-type" }, anomalyDetails.anomaly_type),
                    React.createElement("span", { className: "confidence-score" },
                        "Confidence: ",
                        (anomalyDetails.confidence_score * 100).toFixed(1),
                        "%")),
                React.createElement("div", { className: "anomaly-reasons" },
                    React.createElement("strong", null, "Reasons:"),
                    React.createElement("ul", null, anomalyDetails.reasons.map((reason, index) => (React.createElement("li", { key: index }, reason))))),
                anomalyDetails.baseline_metrics && (React.createElement("div", { className: "metrics-comparison" },
                    React.createElement("div", { className: "metrics-column" },
                        React.createElement("strong", null, "Baseline Metrics:"),
                        React.createElement("pre", null, JSON.stringify(anomalyDetails.baseline_metrics, null, 2))),
                    React.createElement("div", { className: "metrics-column" },
                        React.createElement("strong", null, "Current Metrics:"),
                        React.createElement("pre", null, JSON.stringify(anomalyDetails.current_metrics, null, 2))))))))));
}
function MetadataTab({ event }) {
    return (React.createElement("div", { className: "metadata-tab" },
        React.createElement("section", { className: "detail-section" },
            React.createElement("h3", null, "Custom Metadata"),
            React.createElement("div", { className: "metadata-content" },
                React.createElement("pre", { className: "json-viewer" }, JSON.stringify(event.metadata, null, 2)))),
        React.createElement("section", { className: "detail-section" },
            React.createElement("h3", null, "Compliance & Classification"),
            React.createElement("div", { className: "detail-grid" },
                React.createElement(DetailField, { label: "Data Classification", value: event.data_classification || 'N/A' }),
                React.createElement(DetailField, { label: "Retention Policy", value: event.retention_policy }),
                React.createElement(DetailField, { label: "Compliance Frameworks", value: event.compliance_frameworks?.join(', ') || 'N/A', fullWidth: true }))),
        React.createElement("section", { className: "detail-section" },
            React.createElement("h3", null, "Organization & Tenant"),
            React.createElement("div", { className: "detail-grid" },
                React.createElement(DetailField, { label: "Organization ID", value: event.organization_id || 'N/A', copyable: true }),
                React.createElement(DetailField, { label: "Tenant ID", value: event.tenant_id || 'N/A', copyable: true })))));
}
function ChangesTab({ changes, }) {
    return (React.createElement("div", { className: "changes-tab" },
        React.createElement("section", { className: "detail-section" },
            React.createElement("h3", null, "Data Changes"),
            React.createElement("table", { className: "changes-table" },
                React.createElement("thead", null,
                    React.createElement("tr", null,
                        React.createElement("th", null, "Field"),
                        React.createElement("th", null, "Old Value"),
                        React.createElement("th", null, "New Value"))),
                React.createElement("tbody", null, changes.map((change, index) => (React.createElement("tr", { key: index },
                    React.createElement("td", { className: "field-name" }, change.field),
                    React.createElement("td", { className: "old-value" },
                        React.createElement("code", null, formatValue(change.old_value))),
                    React.createElement("td", { className: "new-value" },
                        React.createElement("code", null, formatValue(change.new_value)))))))))));
}
function SecurityTab({ event, anomalyDetails, }) {
    return (React.createElement("div", { className: "security-tab" },
        React.createElement("section", { className: "detail-section" },
            React.createElement("h3", null, "Security Assessment"),
            React.createElement("div", { className: "detail-grid" },
                React.createElement(DetailField, { label: "Risk Score", value: event.risk_score?.toFixed(0) || 'N/A' }),
                React.createElement(DetailField, { label: "Anomaly Detected", value: event.anomaly_detected ? 'Yes' : 'No' }),
                anomalyDetails && (React.createElement(DetailField, { label: "Anomaly Type", value: anomalyDetails.anomaly_type || 'N/A', fullWidth: true })))),
        React.createElement("section", { className: "detail-section" },
            React.createElement("h3", null, "Cryptographic Integrity"),
            React.createElement("div", { className: "detail-grid" },
                React.createElement(DetailField, { label: "Event Hash", value: event.hash, copyable: true, fullWidth: true }),
                React.createElement(DetailField, { label: "Digital Signature", value: event.signature, copyable: true, fullWidth: true }),
                React.createElement(DetailField, { label: "Previous Event Hash", value: event.previous_event_hash || 'N/A (First Event)', copyable: true, fullWidth: true })),
            React.createElement("div", { className: "integrity-info" },
                React.createElement("p", null, "This audit event is part of a tamper-proof hash chain. Each event contains the hash of the previous event, ensuring integrity of the entire audit trail."))),
        React.createElement("section", { className: "detail-section" },
            React.createElement("h3", null, "Session Details"),
            React.createElement("div", { className: "detail-grid" },
                React.createElement(DetailField, { label: "Session ID", value: event.session_id, copyable: true }),
                React.createElement(DetailField, { label: "IP Address", value: event.user_ip_address, copyable: true }),
                React.createElement(DetailField, { label: "User Agent", value: event.user_agent, fullWidth: true, truncate: true })))));
}
function DetailField({ label, value, copyable = false, fullWidth = false, truncate = false, }) {
    const [copied, setCopied] = useState(false);
    const handleCopy = () => {
        navigator.clipboard.writeText(value);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };
    return (React.createElement("div", { className: `detail-field ${fullWidth ? 'full-width' : ''}` },
        React.createElement("label", null, label),
        React.createElement("div", { className: "field-value" },
            React.createElement("span", { className: truncate ? 'truncate' : '' }, value),
            copyable && (React.createElement("button", { className: "copy-button", onClick: handleCopy, title: copied ? 'Copied!' : 'Copy to clipboard' }, copied ? '✓' : '📋')))));
}
function SeverityBadge({ severity }) {
    const colors = {
        low: 'green',
        medium: 'yellow',
        high: 'orange',
        critical: 'red',
    };
    return (React.createElement("span", { className: `badge badge-${colors[severity] || 'gray'}` }, severity.toUpperCase()));
}
function StatusBadge({ status }) {
    const colors = {
        success: 'green',
        failure: 'red',
        pending: 'yellow',
        blocked: 'gray',
    };
    return (React.createElement("span", { className: `badge badge-${colors[status] || 'gray'}` }, status.toUpperCase()));
}
function formatTimestamp(timestamp) {
    const date = new Date(timestamp);
    return new Intl.DateTimeFormat('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        timeZoneName: 'short',
    }).format(date);
}
function formatEventType(eventType) {
    return eventType
        .split('.')
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join(' ');
}
function formatValue(value) {
    if (value === null || value === undefined)
        return 'null';
    if (typeof value === 'object')
        return JSON.stringify(value);
    return String(value);
}
//# sourceMappingURL=AuditDetail.js.map