import React, { useEffect, useState } from 'react';
export const MaintenanceMode = ({ className = '' }) => {
    const [windows, setWindows] = useState([]);
    const [loading, setLoading] = useState(true);
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [formData, setFormData] = useState({
        title: '',
        description: '',
        services: [],
        startTime: new Date(),
        endTime: new Date(Date.now() + 3600000),
        impactLevel: 'minor',
        notifyUsers: true
    });
    useEffect(() => {
        fetchMaintenanceWindows();
    }, []);
    const fetchMaintenanceWindows = async () => {
        try {
            setLoading(true);
            const response = await fetch('/api/monitoring/maintenance');
            if (!response.ok)
                throw new Error('Failed to fetch maintenance windows');
            const data = await response.json();
            setWindows(data);
        }
        catch (error) {
            console.error('[MaintenanceMode] Failed to fetch windows:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const createWindow = async () => {
        try {
            const response = await fetch('/api/monitoring/maintenance', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    ...formData,
                    status: 'scheduled',
                    createdBy: 'current-user'
                })
            });
            if (!response.ok)
                throw new Error('Failed to create maintenance window');
            const newWindow = await response.json();
            setWindows(prev => [...prev, newWindow]);
            setShowCreateModal(false);
            resetForm();
        }
        catch (error) {
            console.error('[MaintenanceMode] Failed to create window:', error);
            alert('Failed to create maintenance window');
        }
    };
    const updateWindow = async (id, updates) => {
        try {
            const response = await fetch(`/api/monitoring/maintenance/${id}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(updates)
            });
            if (!response.ok)
                throw new Error('Failed to update maintenance window');
            const updatedWindow = await response.json();
            setWindows(prev => prev.map(w => w.id === id ? updatedWindow : w));
        }
        catch (error) {
            console.error('[MaintenanceMode] Failed to update window:', error);
            alert('Failed to update maintenance window');
        }
    };
    const cancelWindow = async (id) => {
        if (!confirm('Are you sure you want to cancel this maintenance window?'))
            return;
        await updateWindow(id, { status: 'cancelled' });
    };
    const startMaintenance = async (id) => {
        if (!confirm('Start this maintenance window now?'))
            return;
        await updateWindow(id, { status: 'active' });
    };
    const completeMaintenance = async (id) => {
        if (!confirm('Mark this maintenance window as completed?'))
            return;
        await updateWindow(id, { status: 'completed' });
    };
    const resetForm = () => {
        setFormData({
            title: '',
            description: '',
            services: [],
            startTime: new Date(),
            endTime: new Date(Date.now() + 3600000),
            impactLevel: 'minor',
            notifyUsers: true
        });
    };
    const handleSubmit = (e) => {
        e.preventDefault();
        if (!formData.title || !formData.description || !formData.services?.length) {
            alert('Please fill in all required fields');
            return;
        }
        createWindow();
    };
    const getStatusColor = (status) => {
        switch (status) {
            case 'scheduled':
                return '#3b82f6';
            case 'active':
                return '#f59e0b';
            case 'completed':
                return '#10b981';
            case 'cancelled':
                return '#6b7280';
            default:
                return '#9ca3af';
        }
    };
    const getImpactColor = (impact) => {
        switch (impact) {
            case 'none':
                return '#10b981';
            case 'minor':
                return '#3b82f6';
            case 'major':
                return '#f59e0b';
            case 'full':
                return '#ef4444';
            default:
                return '#6b7280';
        }
    };
    const formatDuration = (start, end) => {
        const ms = new Date(end).getTime() - new Date(start).getTime();
        const hours = Math.floor(ms / 3600000);
        const minutes = Math.floor((ms % 3600000) / 60000);
        if (hours > 0) {
            return `${hours}h ${minutes}m`;
        }
        return `${minutes}m`;
    };
    const isUpcoming = (window) => {
        return window.status === 'scheduled' && new Date(window.startTime) > new Date();
    };
    const isActive = (window) => {
        return window.status === 'active';
    };
    const upcomingWindows = windows.filter(isUpcoming).sort((a, b) => new Date(a.startTime).getTime() - new Date(b.startTime).getTime());
    const activeWindows = windows.filter(isActive);
    const pastWindows = windows.filter(w => w.status === 'completed' || w.status === 'cancelled').sort((a, b) => new Date(b.startTime).getTime() - new Date(a.startTime).getTime());
    if (loading) {
        return (React.createElement("div", { style: styles.loading },
            React.createElement("div", { style: styles.spinner }),
            React.createElement("p", null, "Loading maintenance windows...")));
    }
    return (React.createElement("div", { className: `maintenance-mode ${className}`, style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("div", null,
                React.createElement("h2", { style: styles.title }, "Maintenance Management"),
                React.createElement("p", { style: styles.subtitle }, "Schedule and manage maintenance windows")),
            React.createElement("button", { style: styles.createButton, onClick: () => {
                    resetForm();
                    setShowCreateModal(true);
                } }, "+ Schedule Maintenance")),
        React.createElement("div", { style: styles.stats },
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: { ...styles.statValue, color: '#f59e0b' } }, activeWindows.length),
                React.createElement("div", { style: styles.statLabel }, "Active")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: { ...styles.statValue, color: '#3b82f6' } }, upcomingWindows.length),
                React.createElement("div", { style: styles.statLabel }, "Upcoming")),
            React.createElement("div", { style: styles.statCard },
                React.createElement("div", { style: styles.statValue }, windows.length),
                React.createElement("div", { style: styles.statLabel }, "Total"))),
        activeWindows.length > 0 && (React.createElement("div", { style: styles.section },
            React.createElement("h3", { style: styles.sectionTitle }, "\uD83D\uDD27 Active Maintenance"),
            activeWindows.map(window => (React.createElement("div", { key: window.id, style: { ...styles.windowCard, borderColor: '#f59e0b', borderWidth: '2px' } },
                React.createElement("div", { style: styles.windowHeader },
                    React.createElement("div", null,
                        React.createElement("h4", { style: styles.windowTitle }, window.title),
                        React.createElement("p", { style: styles.windowDescription }, window.description)),
                    React.createElement("div", { style: styles.windowActions },
                        React.createElement("button", { style: styles.actionButton, onClick: () => completeMaintenance(window.id) }, "Complete"))),
                React.createElement("div", { style: styles.windowDetails },
                    React.createElement("div", { style: styles.detailItem },
                        React.createElement("strong", null, "Started:"),
                        " ",
                        new Date(window.startTime).toLocaleString()),
                    React.createElement("div", { style: styles.detailItem },
                        React.createElement("strong", null, "Expected End:"),
                        " ",
                        new Date(window.endTime).toLocaleString()),
                    React.createElement("div", { style: styles.detailItem },
                        React.createElement("strong", null, "Duration:"),
                        " ",
                        formatDuration(window.startTime, window.endTime)),
                    React.createElement("div", { style: styles.detailItem },
                        React.createElement("strong", null, "Impact:"),
                        React.createElement("span", { style: {
                                ...styles.impactBadge,
                                backgroundColor: `${getImpactColor(window.impactLevel)}20`,
                                color: getImpactColor(window.impactLevel)
                            } }, window.impactLevel.toUpperCase()))),
                window.services.length > 0 && (React.createElement("div", { style: styles.services },
                    React.createElement("strong", null, "Affected Services:"),
                    React.createElement("div", { style: styles.servicesList }, window.services.map((svc, idx) => (React.createElement("span", { key: idx, style: styles.serviceTag }, svc))))))))))),
        upcomingWindows.length > 0 && (React.createElement("div", { style: styles.section },
            React.createElement("h3", { style: styles.sectionTitle }, "\uD83D\uDCC5 Upcoming Maintenance"),
            React.createElement("div", { style: styles.windowsList }, upcomingWindows.map(window => (React.createElement("div", { key: window.id, style: styles.windowCard },
                React.createElement("div", { style: styles.windowHeader },
                    React.createElement("div", null,
                        React.createElement("h4", { style: styles.windowTitle }, window.title),
                        React.createElement("p", { style: styles.windowDescription }, window.description)),
                    React.createElement("div", { style: styles.windowActions },
                        React.createElement("button", { style: styles.actionButton, onClick: () => startMaintenance(window.id) }, "Start Now"),
                        React.createElement("button", { style: { ...styles.actionButton, ...styles.cancelButton }, onClick: () => cancelWindow(window.id) }, "Cancel"))),
                React.createElement("div", { style: styles.windowDetails },
                    React.createElement("div", { style: styles.detailItem },
                        React.createElement("strong", null, "Starts:"),
                        " ",
                        new Date(window.startTime).toLocaleString()),
                    React.createElement("div", { style: styles.detailItem },
                        React.createElement("strong", null, "Ends:"),
                        " ",
                        new Date(window.endTime).toLocaleString()),
                    React.createElement("div", { style: styles.detailItem },
                        React.createElement("strong", null, "Duration:"),
                        " ",
                        formatDuration(window.startTime, window.endTime)),
                    React.createElement("div", { style: styles.detailItem },
                        React.createElement("strong", null, "Impact:"),
                        React.createElement("span", { style: {
                                ...styles.impactBadge,
                                backgroundColor: `${getImpactColor(window.impactLevel)}20`,
                                color: getImpactColor(window.impactLevel)
                            } }, window.impactLevel.toUpperCase())),
                    React.createElement("div", { style: styles.detailItem },
                        React.createElement("strong", null, "Notify Users:"),
                        " ",
                        window.notifyUsers ? 'Yes' : 'No')),
                window.services.length > 0 && (React.createElement("div", { style: styles.services },
                    React.createElement("strong", null, "Affected Services:"),
                    React.createElement("div", { style: styles.servicesList }, window.services.map((svc, idx) => (React.createElement("span", { key: idx, style: styles.serviceTag }, svc)))))))))))),
        pastWindows.length > 0 && (React.createElement("div", { style: styles.section },
            React.createElement("h3", { style: styles.sectionTitle }, "\uD83D\uDCDC Past Maintenance"),
            React.createElement("div", { style: styles.windowsList }, pastWindows.slice(0, 5).map(window => (React.createElement("div", { key: window.id, style: styles.windowCard },
                React.createElement("div", { style: styles.windowHeader },
                    React.createElement("div", null,
                        React.createElement("h4", { style: styles.windowTitle },
                            window.title,
                            React.createElement("span", { style: {
                                    ...styles.statusBadge,
                                    backgroundColor: `${getStatusColor(window.status)}20`,
                                    color: getStatusColor(window.status)
                                } }, window.status.toUpperCase())),
                        React.createElement("p", { style: styles.windowDescription }, window.description))),
                React.createElement("div", { style: styles.windowDetails },
                    React.createElement("div", { style: styles.detailItem },
                        React.createElement("strong", null, "Date:"),
                        " ",
                        new Date(window.startTime).toLocaleString()),
                    React.createElement("div", { style: styles.detailItem },
                        React.createElement("strong", null, "Duration:"),
                        " ",
                        formatDuration(window.startTime, window.endTime)),
                    React.createElement("div", { style: styles.detailItem },
                        React.createElement("strong", null, "Services:"),
                        " ",
                        window.services.join(', '))))))))),
        showCreateModal && (React.createElement("div", { style: styles.modal, onClick: () => setShowCreateModal(false) },
            React.createElement("div", { style: styles.modalContent, onClick: (e) => e.stopPropagation() },
                React.createElement("div", { style: styles.modalHeader },
                    React.createElement("h3", null, "Schedule Maintenance Window"),
                    React.createElement("button", { style: styles.modalClose, onClick: () => setShowCreateModal(false) }, "\u00D7")),
                React.createElement("form", { onSubmit: handleSubmit, style: styles.form },
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Title *"),
                        React.createElement("input", { type: "text", value: formData.title, onChange: (e) => setFormData({ ...formData, title: e.target.value }), style: styles.input, placeholder: "e.g., Database Migration", required: true })),
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Description *"),
                        React.createElement("textarea", { value: formData.description, onChange: (e) => setFormData({ ...formData, description: e.target.value }), style: { ...styles.input, minHeight: '80px' }, placeholder: "Describe the maintenance work...", required: true })),
                    React.createElement("div", { style: styles.formRow },
                        React.createElement("div", { style: styles.formGroup },
                            React.createElement("label", { style: styles.label }, "Start Time *"),
                            React.createElement("input", { type: "datetime-local", value: new Date(formData.startTime).toISOString().slice(0, 16), onChange: (e) => setFormData({ ...formData, startTime: new Date(e.target.value) }), style: styles.input, required: true })),
                        React.createElement("div", { style: styles.formGroup },
                            React.createElement("label", { style: styles.label }, "End Time *"),
                            React.createElement("input", { type: "datetime-local", value: new Date(formData.endTime).toISOString().slice(0, 16), onChange: (e) => setFormData({ ...formData, endTime: new Date(e.target.value) }), style: styles.input, required: true }))),
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Impact Level *"),
                        React.createElement("select", { value: formData.impactLevel, onChange: (e) => setFormData({ ...formData, impactLevel: e.target.value }), style: styles.select },
                            React.createElement("option", { value: "none" }, "None - No user impact"),
                            React.createElement("option", { value: "minor" }, "Minor - Limited functionality"),
                            React.createElement("option", { value: "major" }, "Major - Significant degradation"),
                            React.createElement("option", { value: "full" }, "Full - Complete outage"))),
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.label }, "Affected Services *"),
                        React.createElement("input", { type: "text", value: formData.services?.join(', '), onChange: (e) => setFormData({
                                ...formData,
                                services: e.target.value.split(',').map(s => s.trim()).filter(Boolean)
                            }), style: styles.input, placeholder: "service1, service2, service3", required: true })),
                    React.createElement("div", { style: styles.formGroup },
                        React.createElement("label", { style: styles.checkbox },
                            React.createElement("input", { type: "checkbox", checked: formData.notifyUsers, onChange: (e) => setFormData({ ...formData, notifyUsers: e.target.checked }) }),
                            "Notify users about this maintenance")),
                    React.createElement("div", { style: styles.formActions },
                        React.createElement("button", { type: "button", style: styles.cancelFormButton, onClick: () => setShowCreateModal(false) }, "Cancel"),
                        React.createElement("button", { type: "submit", style: styles.submitButton }, "Schedule Maintenance"))))))));
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
    windowsList: {
        display: 'flex',
        flexDirection: 'column',
        gap: '12px'
    },
    windowCard: {
        backgroundColor: '#fff',
        border: '1px solid #e5e7eb',
        borderRadius: '8px',
        padding: '20px'
    },
    windowHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'flex-start',
        marginBottom: '16px'
    },
    windowTitle: {
        fontSize: '16px',
        fontWeight: 600,
        color: '#111827',
        margin: 0,
        marginBottom: '4px',
        display: 'flex',
        alignItems: 'center',
        gap: '8px'
    },
    windowDescription: {
        fontSize: '14px',
        color: '#6b7280',
        margin: 0
    },
    windowActions: {
        display: 'flex',
        gap: '8px'
    },
    actionButton: {
        padding: '6px 12px',
        backgroundColor: '#3b82f6',
        color: '#fff',
        border: 'none',
        borderRadius: '6px',
        fontSize: '13px',
        fontWeight: 500,
        cursor: 'pointer'
    },
    cancelButton: {
        backgroundColor: '#fff',
        color: '#ef4444',
        border: '1px solid #ef4444'
    },
    windowDetails: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
        gap: '12px',
        marginBottom: '12px'
    },
    detailItem: {
        fontSize: '13px',
        color: '#4b5563',
        display: 'flex',
        alignItems: 'center',
        gap: '8px'
    },
    statusBadge: {
        fontSize: '10px',
        fontWeight: 600,
        padding: '3px 8px',
        borderRadius: '10px',
        marginLeft: '8px'
    },
    impactBadge: {
        fontSize: '11px',
        fontWeight: 600,
        padding: '3px 8px',
        borderRadius: '10px'
    },
    services: {
        paddingTop: '12px',
        borderTop: '1px solid #e5e7eb'
    },
    servicesList: {
        display: 'flex',
        flexWrap: 'wrap',
        gap: '6px',
        marginTop: '8px'
    },
    serviceTag: {
        fontSize: '12px',
        padding: '4px 8px',
        backgroundColor: '#f3f4f6',
        borderRadius: '4px',
        color: '#374151'
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
    cancelFormButton: {
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
export default MaintenanceMode;
//# sourceMappingURL=MaintenanceMode.js.map