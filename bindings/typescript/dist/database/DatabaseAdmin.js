import React, { useState, useEffect } from 'react';
import { useDatabase, useDatabaseStats } from './DatabaseProvider';
export function DatabaseAdmin() {
    const { stats, refetch: refetchStats } = useDatabaseStats(5000);
    const { healthCheck, createBackup, listBackups, restoreBackup, invalidateCache, } = useDatabase();
    const [activeTab, setActiveTab] = useState('overview');
    const [health, setHealth] = useState(null);
    const [backups, setBackups] = useState([]);
    const [isLoadingBackups, setIsLoadingBackups] = useState(false);
    const [isCreatingBackup, setIsCreatingBackup] = useState(false);
    const [message, setMessage] = useState(null);
    useEffect(() => {
        const loadHealth = async () => {
            try {
                const result = await healthCheck();
                setHealth(result);
            }
            catch (err) {
                console.error('Health check failed:', err);
            }
        };
        loadHealth();
        const interval = setInterval(loadHealth, 10000);
        return () => clearInterval(interval);
    }, [healthCheck]);
    const loadBackups = async () => {
        setIsLoadingBackups(true);
        try {
            const result = await listBackups();
            setBackups(result);
        }
        catch (err) {
            showMessage('error', 'Failed to load backups');
        }
        finally {
            setIsLoadingBackups(false);
        }
    };
    useEffect(() => {
        if (activeTab === 'backups') {
            loadBackups();
        }
    }, [activeTab]);
    const handleCreateBackup = async () => {
        setIsCreatingBackup(true);
        try {
            const backupId = await createBackup();
            showMessage('success', `Backup created: ${backupId}`);
            await loadBackups();
        }
        catch (err) {
            showMessage('error', 'Failed to create backup');
        }
        finally {
            setIsCreatingBackup(false);
        }
    };
    const handleRestoreBackup = async (backupId) => {
        if (!confirm(`Are you sure you want to restore backup ${backupId}? This will overwrite the current database.`)) {
            return;
        }
        try {
            await restoreBackup(backupId);
            showMessage('success', 'Backup restored successfully');
        }
        catch (err) {
            showMessage('error', 'Failed to restore backup');
        }
    };
    const handleClearCache = () => {
        if (!confirm('Are you sure you want to clear all cache?')) {
            return;
        }
        invalidateCache();
        showMessage('success', 'Cache cleared');
    };
    const showMessage = (type, text) => {
        setMessage({ type, text });
        setTimeout(() => setMessage(null), 5000);
    };
    const formatBytes = (bytes) => {
        if (bytes === 0)
            return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
    };
    const formatDate = (dateString) => {
        return new Date(dateString).toLocaleString();
    };
    return (React.createElement("div", { style: styles.container },
        React.createElement("h1", { style: styles.title }, "Database Administration"),
        message && (React.createElement("div", { style: {
                ...styles.message,
                backgroundColor: message.type === 'success' ? '#d4edda' : '#f8d7da',
                color: message.type === 'success' ? '#155724' : '#721c24',
            } }, message.text)),
        React.createElement("div", { style: styles.tabs },
            React.createElement("button", { style: {
                    ...styles.tab,
                    ...(activeTab === 'overview' ? styles.activeTab : {}),
                }, onClick: () => setActiveTab('overview') }, "Overview"),
            React.createElement("button", { style: {
                    ...styles.tab,
                    ...(activeTab === 'backups' ? styles.activeTab : {}),
                }, onClick: () => setActiveTab('backups') }, "Backups"),
            React.createElement("button", { style: {
                    ...styles.tab,
                    ...(activeTab === 'cache' ? styles.activeTab : {}),
                }, onClick: () => setActiveTab('cache') }, "Cache"),
            React.createElement("button", { style: {
                    ...styles.tab,
                    ...(activeTab === 'health' ? styles.activeTab : {}),
                }, onClick: () => setActiveTab('health') }, "Health")),
        React.createElement("div", { style: styles.content },
            activeTab === 'overview' && stats && (React.createElement("div", { style: styles.section },
                React.createElement("h2", { style: styles.sectionTitle }, "Database Overview"),
                React.createElement("div", { style: styles.grid },
                    React.createElement("div", { style: styles.card },
                        React.createElement("h3", { style: styles.cardTitle }, "Connection Pool"),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Active Connections:"),
                            React.createElement("span", { style: styles.statValue }, stats.pool.activeConnections)),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Idle Connections:"),
                            React.createElement("span", { style: styles.statValue }, stats.pool.idleConnections)),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Total Queries:"),
                            React.createElement("span", { style: styles.statValue }, stats.pool.totalQueries.toLocaleString())),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Avg Query Time:"),
                            React.createElement("span", { style: styles.statValue },
                                stats.pool.avgQueryTimeUs.toFixed(2),
                                " \u03BCs")),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Status:"),
                            React.createElement("span", { style: {
                                    ...styles.statValue,
                                    color: stats.pool.isHealthy ? '#28a745' : '#dc3545',
                                } }, stats.pool.isHealthy ? 'Healthy' : 'Unhealthy'))),
                    React.createElement("div", { style: styles.card },
                        React.createElement("h3", { style: styles.cardTitle }, "Cache Performance"),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Hit Rate:"),
                            React.createElement("span", { style: styles.statValue },
                                (stats.cache.hitRate * 100).toFixed(2),
                                "%")),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Total Hits:"),
                            React.createElement("span", { style: styles.statValue }, stats.cache.totalHits.toLocaleString())),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Total Misses:"),
                            React.createElement("span", { style: styles.statValue }, stats.cache.totalMisses.toLocaleString())),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "L1 Size:"),
                            React.createElement("span", { style: styles.statValue }, stats.cache.l1Size)),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "L2 Size:"),
                            React.createElement("span", { style: styles.statValue }, formatBytes(stats.cache.l2Size)))),
                    stats.replication && (React.createElement("div", { style: styles.card },
                        React.createElement("h3", { style: styles.cardTitle }, "Replication"),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Replicas:"),
                            React.createElement("span", { style: styles.statValue },
                                stats.replication.healthyReplicas,
                                " / ",
                                stats.replication.replicaCount)),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Avg Lag:"),
                            React.createElement("span", { style: styles.statValue },
                                stats.replication.avgLagMs,
                                " ms")),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Success Rate:"),
                            React.createElement("span", { style: styles.statValue },
                                (stats.replication.successRate * 100).toFixed(2),
                                "%")))),
                    stats.sharding && (React.createElement("div", { style: styles.card },
                        React.createElement("h3", { style: styles.cardTitle }, "Sharding"),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Shards:"),
                            React.createElement("span", { style: styles.statValue },
                                stats.sharding.availableShards,
                                " / ",
                                stats.sharding.totalShards)),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Total Lookups:"),
                            React.createElement("span", { style: styles.statValue }, stats.sharding.totalLookups.toLocaleString())),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Cross-Shard Queries:"),
                            React.createElement("span", { style: styles.statValue }, stats.sharding.crossShardQueries))))))),
            activeTab === 'backups' && (React.createElement("div", { style: styles.section },
                React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' } },
                    React.createElement("h2", { style: styles.sectionTitle }, "Backups"),
                    React.createElement("button", { style: styles.button, onClick: handleCreateBackup, disabled: isCreatingBackup }, isCreatingBackup ? 'Creating...' : 'Create Backup')),
                isLoadingBackups ? (React.createElement("div", { style: styles.loading }, "Loading backups...")) : backups.length === 0 ? (React.createElement("div", { style: styles.empty }, "No backups found")) : (React.createElement("table", { style: styles.table },
                    React.createElement("thead", null,
                        React.createElement("tr", null,
                            React.createElement("th", { style: styles.th }, "ID"),
                            React.createElement("th", { style: styles.th }, "Type"),
                            React.createElement("th", { style: styles.th }, "Created"),
                            React.createElement("th", { style: styles.th }, "Size"),
                            React.createElement("th", { style: styles.th }, "Compressed"),
                            React.createElement("th", { style: styles.th }, "Actions"))),
                    React.createElement("tbody", null, backups.map((backup) => (React.createElement("tr", { key: backup.id },
                        React.createElement("td", { style: styles.td }, backup.id),
                        React.createElement("td", { style: styles.td }, backup.type),
                        React.createElement("td", { style: styles.td }, formatDate(backup.createdAt)),
                        React.createElement("td", { style: styles.td }, formatBytes(backup.sizeBytes)),
                        React.createElement("td", { style: styles.td }, backup.compressedSize
                            ? formatBytes(backup.compressedSize)
                            : '-'),
                        React.createElement("td", { style: styles.td },
                            React.createElement("button", { style: styles.smallButton, onClick: () => handleRestoreBackup(backup.id) }, "Restore")))))))))),
            activeTab === 'cache' && stats && (React.createElement("div", { style: styles.section },
                React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' } },
                    React.createElement("h2", { style: styles.sectionTitle }, "Cache Management"),
                    React.createElement("button", { style: styles.button, onClick: handleClearCache }, "Clear All Cache")),
                React.createElement("div", { style: styles.grid },
                    React.createElement("div", { style: styles.card },
                        React.createElement("h3", { style: styles.cardTitle }, "L1 Cache (Memory)"),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Hits:"),
                            React.createElement("span", { style: styles.statValue }, stats.cache.l1Hits.toLocaleString())),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Misses:"),
                            React.createElement("span", { style: styles.statValue }, stats.cache.l1Misses.toLocaleString())),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Entries:"),
                            React.createElement("span", { style: styles.statValue }, stats.cache.l1Size))),
                    React.createElement("div", { style: styles.card },
                        React.createElement("h3", { style: styles.cardTitle }, "L2 Cache (Disk)"),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Hits:"),
                            React.createElement("span", { style: styles.statValue }, stats.cache.l2Hits.toLocaleString())),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Misses:"),
                            React.createElement("span", { style: styles.statValue }, stats.cache.l2Misses.toLocaleString())),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Size:"),
                            React.createElement("span", { style: styles.statValue }, formatBytes(stats.cache.l2Size)))),
                    React.createElement("div", { style: styles.card },
                        React.createElement("h3", { style: styles.cardTitle }, "L3 Cache (Distributed)"),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Hits:"),
                            React.createElement("span", { style: styles.statValue }, stats.cache.l3Hits.toLocaleString())),
                        React.createElement("div", { style: styles.stat },
                            React.createElement("span", null, "Misses:"),
                            React.createElement("span", { style: styles.statValue }, stats.cache.l3Misses.toLocaleString())))))),
            activeTab === 'health' && health && (React.createElement("div", { style: styles.section },
                React.createElement("h2", { style: styles.sectionTitle }, "Health Status"),
                React.createElement("div", { style: styles.card },
                    React.createElement("div", { style: styles.stat },
                        React.createElement("span", null, "Overall Status:"),
                        React.createElement("span", { style: {
                                ...styles.statValue,
                                color: health.healthy ? '#28a745' : '#dc3545',
                                fontWeight: 'bold',
                            } }, health.healthy ? 'HEALTHY' : 'UNHEALTHY')),
                    React.createElement("div", { style: styles.stat },
                        React.createElement("span", null, "Response Time:"),
                        React.createElement("span", { style: styles.statValue },
                            health.responseTime,
                            " ms")),
                    React.createElement("div", { style: styles.stat },
                        React.createElement("span", null, "Timestamp:"),
                        React.createElement("span", { style: styles.statValue }, formatDate(health.timestamp)))),
                React.createElement("h3", { style: { ...styles.sectionTitle, marginTop: '30px' } }, "Components"),
                React.createElement("div", { style: styles.grid }, Object.entries(health.components).map(([component, status]) => (React.createElement("div", { key: component, style: styles.card },
                    React.createElement("h4", { style: styles.cardTitle }, component.charAt(0).toUpperCase() + component.slice(1)),
                    React.createElement("div", { style: styles.stat },
                        React.createElement("span", null, "Status:"),
                        React.createElement("span", { style: {
                                ...styles.statValue,
                                color: status ? '#28a745' : '#dc3545',
                            } }, status ? 'OK' : 'ERROR')))))),
                health.errors && health.errors.length > 0 && (React.createElement(React.Fragment, null,
                    React.createElement("h3", { style: { ...styles.sectionTitle, marginTop: '30px' } }, "Errors"),
                    React.createElement("div", { style: styles.errorList }, health.errors.map((error, index) => (React.createElement("div", { key: index, style: styles.error }, error)))))))))));
}
const styles = {
    container: {
        padding: '20px',
        fontFamily: 'system-ui, -apple-system, sans-serif',
        maxWidth: '1400px',
        margin: '0 auto',
    },
    title: {
        fontSize: '28px',
        fontWeight: 'bold',
        marginBottom: '20px',
        color: '#333',
    },
    message: {
        padding: '12px 16px',
        borderRadius: '4px',
        marginBottom: '20px',
        border: '1px solid',
    },
    tabs: {
        display: 'flex',
        borderBottom: '2px solid #e0e0e0',
        marginBottom: '20px',
    },
    tab: {
        padding: '12px 24px',
        background: 'none',
        border: 'none',
        cursor: 'pointer',
        fontSize: '16px',
        color: '#666',
        borderBottom: '2px solid transparent',
        marginBottom: '-2px',
    },
    activeTab: {
        color: '#007bff',
        borderBottomColor: '#007bff',
        fontWeight: '500',
    },
    content: {
        marginTop: '20px',
    },
    section: {
        marginBottom: '30px',
    },
    sectionTitle: {
        fontSize: '22px',
        fontWeight: '600',
        marginBottom: '15px',
        color: '#333',
    },
    grid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))',
        gap: '20px',
    },
    card: {
        background: '#fff',
        border: '1px solid #e0e0e0',
        borderRadius: '8px',
        padding: '20px',
        boxShadow: '0 2px 4px rgba(0,0,0,0.05)',
    },
    cardTitle: {
        fontSize: '16px',
        fontWeight: '600',
        marginBottom: '15px',
        color: '#555',
        borderBottom: '1px solid #f0f0f0',
        paddingBottom: '10px',
    },
    stat: {
        display: 'flex',
        justifyContent: 'space-between',
        padding: '8px 0',
        fontSize: '14px',
        color: '#666',
    },
    statValue: {
        fontWeight: '500',
        color: '#333',
    },
    button: {
        padding: '10px 20px',
        background: '#007bff',
        color: '#fff',
        border: 'none',
        borderRadius: '4px',
        cursor: 'pointer',
        fontSize: '14px',
        fontWeight: '500',
    },
    smallButton: {
        padding: '6px 12px',
        background: '#28a745',
        color: '#fff',
        border: 'none',
        borderRadius: '4px',
        cursor: 'pointer',
        fontSize: '12px',
    },
    table: {
        width: '100%',
        borderCollapse: 'collapse',
        background: '#fff',
        borderRadius: '8px',
        overflow: 'hidden',
    },
    th: {
        padding: '12px',
        background: '#f8f9fa',
        textAlign: 'left',
        fontWeight: '600',
        fontSize: '14px',
        color: '#555',
        borderBottom: '2px solid #e0e0e0',
    },
    td: {
        padding: '12px',
        borderBottom: '1px solid #f0f0f0',
        fontSize: '14px',
        color: '#666',
    },
    loading: {
        textAlign: 'center',
        padding: '40px',
        fontSize: '16px',
        color: '#999',
    },
    empty: {
        textAlign: 'center',
        padding: '40px',
        fontSize: '16px',
        color: '#999',
        background: '#f8f9fa',
        borderRadius: '8px',
    },
    errorList: {
        background: '#fff',
        border: '1px solid #e0e0e0',
        borderRadius: '8px',
        padding: '10px',
    },
    error: {
        padding: '10px',
        background: '#f8d7da',
        color: '#721c24',
        borderRadius: '4px',
        marginBottom: '8px',
        fontSize: '14px',
    },
};
//# sourceMappingURL=DatabaseAdmin.js.map