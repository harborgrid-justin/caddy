import React, { useState, useEffect } from 'react';
import { useDatabase } from './DatabaseProvider';
export function MigrationPanel() {
    const { getMigrationStatus, runMigrations } = useDatabase();
    const [status, setStatus] = useState(null);
    const [isLoading, setIsLoading] = useState(true);
    const [isRunning, setIsRunning] = useState(false);
    const [error, setError] = useState(null);
    const [selectedMigration, setSelectedMigration] = useState(null);
    const loadStatus = async () => {
        setIsLoading(true);
        setError(null);
        try {
            const result = await getMigrationStatus();
            setStatus(result);
        }
        catch (err) {
            const message = err instanceof Error ? err.message : 'Failed to load migration status';
            setError(message);
        }
        finally {
            setIsLoading(false);
        }
    };
    useEffect(() => {
        loadStatus();
    }, []);
    const handleRunMigrations = async () => {
        if (!confirm(`Are you sure you want to run ${status?.pending} pending migration(s)?`)) {
            return;
        }
        setIsRunning(true);
        setError(null);
        try {
            await runMigrations();
            await loadStatus();
        }
        catch (err) {
            const message = err instanceof Error ? err.message : 'Migration failed';
            setError(message);
        }
        finally {
            setIsRunning(false);
        }
    };
    const formatDate = (dateString) => {
        if (!dateString)
            return '-';
        return new Date(dateString).toLocaleString();
    };
    if (isLoading) {
        return (React.createElement("div", { style: styles.container },
            React.createElement("div", { style: styles.loading }, "Loading migrations...")));
    }
    if (error && !status) {
        return (React.createElement("div", { style: styles.container },
            React.createElement("div", { style: styles.error },
                "Error: ",
                error),
            React.createElement("button", { style: styles.button, onClick: loadStatus }, "Retry")));
    }
    if (!status) {
        return null;
    }
    return (React.createElement("div", { style: styles.container },
        React.createElement("h1", { style: styles.title }, "Database Migrations"),
        error && (React.createElement("div", { style: styles.errorBanner }, error)),
        React.createElement("div", { style: styles.summary },
            React.createElement("div", { style: styles.summaryCard },
                React.createElement("div", { style: styles.summaryValue }, status.total),
                React.createElement("div", { style: styles.summaryLabel }, "Total Migrations")),
            React.createElement("div", { style: { ...styles.summaryCard, background: '#d4edda' } },
                React.createElement("div", { style: styles.summaryValue }, status.applied),
                React.createElement("div", { style: styles.summaryLabel }, "Applied")),
            React.createElement("div", { style: { ...styles.summaryCard, background: status.pending > 0 ? '#fff3cd' : '#e9ecef' } },
                React.createElement("div", { style: styles.summaryValue }, status.pending),
                React.createElement("div", { style: styles.summaryLabel }, "Pending"))),
        status.pending > 0 && (React.createElement("div", { style: styles.actions },
            React.createElement("button", { style: styles.primaryButton, onClick: handleRunMigrations, disabled: isRunning }, isRunning ? 'Running Migrations...' : `Run ${status.pending} Pending Migration(s)`))),
        status.pending > 0 && (React.createElement("div", { style: styles.section },
            React.createElement("h2", { style: styles.sectionTitle },
                "Pending Migrations",
                React.createElement("span", { style: styles.badge }, status.pending)),
            React.createElement("div", { style: styles.migrationList }, status.pendingMigrations.map((migration) => (React.createElement("div", { key: migration.version, style: styles.migrationCard, onClick: () => setSelectedMigration(migration) },
                React.createElement("div", { style: styles.migrationHeader },
                    React.createElement("div", null,
                        React.createElement("div", { style: styles.migrationName }, migration.name),
                        React.createElement("div", { style: styles.migrationVersion },
                            "Version: ",
                            migration.version)),
                    React.createElement("div", { style: { ...styles.status, background: '#ffc107' } }, "Pending")),
                React.createElement("div", { style: styles.migrationDescription }, migration.description))))))),
        React.createElement("div", { style: styles.section },
            React.createElement("h2", { style: styles.sectionTitle },
                "Migration History",
                React.createElement("span", { style: styles.badge }, status.applied)),
            status.history.length === 0 ? (React.createElement("div", { style: styles.empty }, "No migrations applied yet")) : (React.createElement("div", { style: styles.migrationList }, status.history.map((migration) => (React.createElement("div", { key: migration.version, style: styles.migrationCard, onClick: () => setSelectedMigration(migration) },
                React.createElement("div", { style: styles.migrationHeader },
                    React.createElement("div", null,
                        React.createElement("div", { style: styles.migrationName }, migration.name),
                        React.createElement("div", { style: styles.migrationVersion },
                            "Version: ",
                            migration.version)),
                    React.createElement("div", { style: { ...styles.status, background: '#28a745' } }, "Applied")),
                React.createElement("div", { style: styles.migrationDescription }, migration.description),
                migration.appliedAt && (React.createElement("div", { style: styles.migrationDate },
                    "Applied: ",
                    formatDate(migration.appliedAt))))))))),
        selectedMigration && (React.createElement("div", { style: styles.modal, onClick: () => setSelectedMigration(null) },
            React.createElement("div", { style: styles.modalContent, onClick: (e) => e.stopPropagation() },
                React.createElement("div", { style: styles.modalHeader },
                    React.createElement("h3", { style: styles.modalTitle }, "Migration Details"),
                    React.createElement("button", { style: styles.closeButton, onClick: () => setSelectedMigration(null) }, "\u00D7")),
                React.createElement("div", { style: styles.modalBody },
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("span", { style: styles.detailLabel }, "Name:"),
                        React.createElement("span", { style: styles.detailValue }, selectedMigration.name)),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("span", { style: styles.detailLabel }, "Version:"),
                        React.createElement("span", { style: styles.detailValue }, selectedMigration.version)),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("span", { style: styles.detailLabel }, "Description:"),
                        React.createElement("span", { style: styles.detailValue }, selectedMigration.description)),
                    React.createElement("div", { style: styles.detailRow },
                        React.createElement("span", { style: styles.detailLabel }, "Status:"),
                        React.createElement("span", { style: {
                                ...styles.status,
                                background: selectedMigration.isApplied ? '#28a745' : '#ffc107',
                            } }, selectedMigration.isApplied ? 'Applied' : 'Pending')),
                    selectedMigration.appliedAt && (React.createElement("div", { style: styles.detailRow },
                        React.createElement("span", { style: styles.detailLabel }, "Applied At:"),
                        React.createElement("span", { style: styles.detailValue }, formatDate(selectedMigration.appliedAt)))),
                    selectedMigration.up && (React.createElement("div", { style: styles.codeSection },
                        React.createElement("div", { style: styles.codeLabel }, "UP SQL:"),
                        React.createElement("pre", { style: styles.code }, selectedMigration.up))),
                    selectedMigration.down && (React.createElement("div", { style: styles.codeSection },
                        React.createElement("div", { style: styles.codeLabel }, "DOWN SQL:"),
                        React.createElement("pre", { style: styles.code }, selectedMigration.down)))))))));
}
const styles = {
    container: {
        padding: '20px',
        fontFamily: 'system-ui, -apple-system, sans-serif',
        maxWidth: '1200px',
        margin: '0 auto',
    },
    title: {
        fontSize: '28px',
        fontWeight: 'bold',
        marginBottom: '30px',
        color: '#333',
    },
    summary: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
        gap: '20px',
        marginBottom: '30px',
    },
    summaryCard: {
        background: '#f8f9fa',
        padding: '24px',
        borderRadius: '8px',
        textAlign: 'center',
        border: '1px solid #e0e0e0',
    },
    summaryValue: {
        fontSize: '36px',
        fontWeight: 'bold',
        color: '#333',
        marginBottom: '8px',
    },
    summaryLabel: {
        fontSize: '14px',
        color: '#666',
        textTransform: 'uppercase',
        letterSpacing: '0.5px',
    },
    actions: {
        marginBottom: '30px',
        textAlign: 'center',
    },
    primaryButton: {
        padding: '14px 32px',
        background: '#007bff',
        color: '#fff',
        border: 'none',
        borderRadius: '6px',
        cursor: 'pointer',
        fontSize: '16px',
        fontWeight: '500',
        boxShadow: '0 2px 8px rgba(0,123,255,0.3)',
    },
    button: {
        padding: '10px 20px',
        background: '#6c757d',
        color: '#fff',
        border: 'none',
        borderRadius: '4px',
        cursor: 'pointer',
        fontSize: '14px',
    },
    section: {
        marginBottom: '40px',
    },
    sectionTitle: {
        fontSize: '20px',
        fontWeight: '600',
        marginBottom: '20px',
        color: '#333',
        display: 'flex',
        alignItems: 'center',
        gap: '10px',
    },
    badge: {
        background: '#007bff',
        color: '#fff',
        padding: '4px 12px',
        borderRadius: '12px',
        fontSize: '14px',
        fontWeight: '500',
    },
    migrationList: {
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
    },
    migrationCard: {
        background: '#fff',
        border: '1px solid #e0e0e0',
        borderRadius: '8px',
        padding: '20px',
        cursor: 'pointer',
        transition: 'all 0.2s',
        boxShadow: '0 1px 3px rgba(0,0,0,0.05)',
    },
    migrationHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'flex-start',
        marginBottom: '10px',
    },
    migrationName: {
        fontSize: '16px',
        fontWeight: '600',
        color: '#333',
        marginBottom: '4px',
    },
    migrationVersion: {
        fontSize: '12px',
        color: '#999',
        fontFamily: 'monospace',
    },
    migrationDescription: {
        fontSize: '14px',
        color: '#666',
        lineHeight: '1.5',
    },
    migrationDate: {
        fontSize: '12px',
        color: '#999',
        marginTop: '10px',
        paddingTop: '10px',
        borderTop: '1px solid #f0f0f0',
    },
    status: {
        padding: '6px 12px',
        borderRadius: '4px',
        fontSize: '12px',
        fontWeight: '500',
        color: '#fff',
    },
    loading: {
        textAlign: 'center',
        padding: '60px 20px',
        fontSize: '16px',
        color: '#999',
    },
    error: {
        background: '#f8d7da',
        color: '#721c24',
        padding: '16px',
        borderRadius: '4px',
        marginBottom: '20px',
        border: '1px solid #f5c6cb',
    },
    errorBanner: {
        background: '#f8d7da',
        color: '#721c24',
        padding: '16px',
        borderRadius: '6px',
        marginBottom: '20px',
        border: '1px solid #f5c6cb',
    },
    empty: {
        textAlign: 'center',
        padding: '60px 20px',
        fontSize: '16px',
        color: '#999',
        background: '#f8f9fa',
        borderRadius: '8px',
        border: '1px solid #e0e0e0',
    },
    modal: {
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        background: 'rgba(0,0,0,0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
    },
    modalContent: {
        background: '#fff',
        borderRadius: '12px',
        maxWidth: '800px',
        width: '90%',
        maxHeight: '90vh',
        overflow: 'auto',
        boxShadow: '0 8px 32px rgba(0,0,0,0.2)',
    },
    modalHeader: {
        padding: '24px',
        borderBottom: '1px solid #e0e0e0',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
    },
    modalTitle: {
        fontSize: '20px',
        fontWeight: '600',
        color: '#333',
        margin: 0,
    },
    closeButton: {
        background: 'none',
        border: 'none',
        fontSize: '32px',
        color: '#999',
        cursor: 'pointer',
        padding: '0',
        width: '32px',
        height: '32px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
    },
    modalBody: {
        padding: '24px',
    },
    detailRow: {
        display: 'flex',
        alignItems: 'flex-start',
        marginBottom: '16px',
        gap: '16px',
    },
    detailLabel: {
        fontSize: '14px',
        fontWeight: '600',
        color: '#666',
        minWidth: '120px',
    },
    detailValue: {
        fontSize: '14px',
        color: '#333',
        flex: 1,
    },
    codeSection: {
        marginTop: '24px',
    },
    codeLabel: {
        fontSize: '12px',
        fontWeight: '600',
        color: '#666',
        textTransform: 'uppercase',
        letterSpacing: '0.5px',
        marginBottom: '8px',
    },
    code: {
        background: '#f8f9fa',
        padding: '16px',
        borderRadius: '6px',
        fontSize: '13px',
        fontFamily: 'monospace',
        overflow: 'auto',
        border: '1px solid #e0e0e0',
        lineHeight: '1.5',
        color: '#333',
    },
};
//# sourceMappingURL=MigrationPanel.js.map