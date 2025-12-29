/**
 * CADDY Database - Admin Panel Component
 *
 * Provides a comprehensive admin interface for database operations,
 * monitoring, and management.
 */

import React, { useState, useEffect } from 'react';
import { useDatabase, useDatabaseStats } from './DatabaseProvider';
import { BackupMetadata, HealthCheckResult } from './types';

/**
 * Database admin panel component
 */
export function DatabaseAdmin() {
  const { stats, refetch: refetchStats } = useDatabaseStats(5000); // Refresh every 5 seconds
  const {
    healthCheck,
    createBackup,
    listBackups,
    restoreBackup,
    invalidateCache,
  } = useDatabase();

  const [activeTab, setActiveTab] = useState<'overview' | 'backups' | 'cache' | 'health'>('overview');
  const [health, setHealth] = useState<HealthCheckResult | null>(null);
  const [backups, setBackups] = useState<BackupMetadata[]>([]);
  const [isLoadingBackups, setIsLoadingBackups] = useState(false);
  const [isCreatingBackup, setIsCreatingBackup] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  // Load health status
  useEffect(() => {
    const loadHealth = async () => {
      try {
        const result = await healthCheck();
        setHealth(result);
      } catch (err) {
        console.error('Health check failed:', err);
      }
    };

    loadHealth();
    const interval = setInterval(loadHealth, 10000); // Every 10 seconds

    return () => clearInterval(interval);
  }, [healthCheck]);

  // Load backups
  const loadBackups = async () => {
    setIsLoadingBackups(true);
    try {
      const result = await listBackups();
      setBackups(result);
    } catch (err) {
      showMessage('error', 'Failed to load backups');
    } finally {
      setIsLoadingBackups(false);
    }
  };

  useEffect(() => {
    if (activeTab === 'backups') {
      loadBackups();
    }
  }, [activeTab]);

  // Create backup
  const handleCreateBackup = async () => {
    setIsCreatingBackup(true);
    try {
      const backupId = await createBackup();
      showMessage('success', `Backup created: ${backupId}`);
      await loadBackups();
    } catch (err) {
      showMessage('error', 'Failed to create backup');
    } finally {
      setIsCreatingBackup(false);
    }
  };

  // Restore backup
  const handleRestoreBackup = async (backupId: string) => {
    if (!confirm(`Are you sure you want to restore backup ${backupId}? This will overwrite the current database.`)) {
      return;
    }

    try {
      await restoreBackup(backupId);
      showMessage('success', 'Backup restored successfully');
    } catch (err) {
      showMessage('error', 'Failed to restore backup');
    }
  };

  // Clear cache
  const handleClearCache = () => {
    if (!confirm('Are you sure you want to clear all cache?')) {
      return;
    }

    invalidateCache();
    showMessage('success', 'Cache cleared');
  };

  // Show message
  const showMessage = (type: 'success' | 'error', text: string) => {
    setMessage({ type, text });
    setTimeout(() => setMessage(null), 5000);
  };

  // Format bytes
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  // Format date
  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleString();
  };

  return (
    <div style={styles.container}>
      <h1 style={styles.title}>Database Administration</h1>

      {/* Message */}
      {message && (
        <div style={{
          ...styles.message,
          backgroundColor: message.type === 'success' ? '#d4edda' : '#f8d7da',
          color: message.type === 'success' ? '#155724' : '#721c24',
        }}>
          {message.text}
        </div>
      )}

      {/* Tabs */}
      <div style={styles.tabs}>
        <button
          style={{
            ...styles.tab,
            ...(activeTab === 'overview' ? styles.activeTab : {}),
          }}
          onClick={() => setActiveTab('overview')}
        >
          Overview
        </button>
        <button
          style={{
            ...styles.tab,
            ...(activeTab === 'backups' ? styles.activeTab : {}),
          }}
          onClick={() => setActiveTab('backups')}
        >
          Backups
        </button>
        <button
          style={{
            ...styles.tab,
            ...(activeTab === 'cache' ? styles.activeTab : {}),
          }}
          onClick={() => setActiveTab('cache')}
        >
          Cache
        </button>
        <button
          style={{
            ...styles.tab,
            ...(activeTab === 'health' ? styles.activeTab : {}),
          }}
          onClick={() => setActiveTab('health')}
        >
          Health
        </button>
      </div>

      {/* Tab content */}
      <div style={styles.content}>
        {/* Overview Tab */}
        {activeTab === 'overview' && stats && (
          <div style={styles.section}>
            <h2 style={styles.sectionTitle}>Database Overview</h2>

            <div style={styles.grid}>
              {/* Connection Pool */}
              <div style={styles.card}>
                <h3 style={styles.cardTitle}>Connection Pool</h3>
                <div style={styles.stat}>
                  <span>Active Connections:</span>
                  <span style={styles.statValue}>{stats.pool.activeConnections}</span>
                </div>
                <div style={styles.stat}>
                  <span>Idle Connections:</span>
                  <span style={styles.statValue}>{stats.pool.idleConnections}</span>
                </div>
                <div style={styles.stat}>
                  <span>Total Queries:</span>
                  <span style={styles.statValue}>{stats.pool.totalQueries.toLocaleString()}</span>
                </div>
                <div style={styles.stat}>
                  <span>Avg Query Time:</span>
                  <span style={styles.statValue}>{stats.pool.avgQueryTimeUs.toFixed(2)} μs</span>
                </div>
                <div style={styles.stat}>
                  <span>Status:</span>
                  <span style={{
                    ...styles.statValue,
                    color: stats.pool.isHealthy ? '#28a745' : '#dc3545',
                  }}>
                    {stats.pool.isHealthy ? 'Healthy' : 'Unhealthy'}
                  </span>
                </div>
              </div>

              {/* Cache Stats */}
              <div style={styles.card}>
                <h3 style={styles.cardTitle}>Cache Performance</h3>
                <div style={styles.stat}>
                  <span>Hit Rate:</span>
                  <span style={styles.statValue}>{(stats.cache.hitRate * 100).toFixed(2)}%</span>
                </div>
                <div style={styles.stat}>
                  <span>Total Hits:</span>
                  <span style={styles.statValue}>{stats.cache.totalHits.toLocaleString()}</span>
                </div>
                <div style={styles.stat}>
                  <span>Total Misses:</span>
                  <span style={styles.statValue}>{stats.cache.totalMisses.toLocaleString()}</span>
                </div>
                <div style={styles.stat}>
                  <span>L1 Size:</span>
                  <span style={styles.statValue}>{stats.cache.l1Size}</span>
                </div>
                <div style={styles.stat}>
                  <span>L2 Size:</span>
                  <span style={styles.statValue}>{formatBytes(stats.cache.l2Size)}</span>
                </div>
              </div>

              {/* Replication Stats */}
              {stats.replication && (
                <div style={styles.card}>
                  <h3 style={styles.cardTitle}>Replication</h3>
                  <div style={styles.stat}>
                    <span>Replicas:</span>
                    <span style={styles.statValue}>
                      {stats.replication.healthyReplicas} / {stats.replication.replicaCount}
                    </span>
                  </div>
                  <div style={styles.stat}>
                    <span>Avg Lag:</span>
                    <span style={styles.statValue}>{stats.replication.avgLagMs} ms</span>
                  </div>
                  <div style={styles.stat}>
                    <span>Success Rate:</span>
                    <span style={styles.statValue}>
                      {(stats.replication.successRate * 100).toFixed(2)}%
                    </span>
                  </div>
                </div>
              )}

              {/* Sharding Stats */}
              {stats.sharding && (
                <div style={styles.card}>
                  <h3 style={styles.cardTitle}>Sharding</h3>
                  <div style={styles.stat}>
                    <span>Shards:</span>
                    <span style={styles.statValue}>
                      {stats.sharding.availableShards} / {stats.sharding.totalShards}
                    </span>
                  </div>
                  <div style={styles.stat}>
                    <span>Total Lookups:</span>
                    <span style={styles.statValue}>{stats.sharding.totalLookups.toLocaleString()}</span>
                  </div>
                  <div style={styles.stat}>
                    <span>Cross-Shard Queries:</span>
                    <span style={styles.statValue}>{stats.sharding.crossShardQueries}</span>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Backups Tab */}
        {activeTab === 'backups' && (
          <div style={styles.section}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
              <h2 style={styles.sectionTitle}>Backups</h2>
              <button
                style={styles.button}
                onClick={handleCreateBackup}
                disabled={isCreatingBackup}
              >
                {isCreatingBackup ? 'Creating...' : 'Create Backup'}
              </button>
            </div>

            {isLoadingBackups ? (
              <div style={styles.loading}>Loading backups...</div>
            ) : backups.length === 0 ? (
              <div style={styles.empty}>No backups found</div>
            ) : (
              <table style={styles.table}>
                <thead>
                  <tr>
                    <th style={styles.th}>ID</th>
                    <th style={styles.th}>Type</th>
                    <th style={styles.th}>Created</th>
                    <th style={styles.th}>Size</th>
                    <th style={styles.th}>Compressed</th>
                    <th style={styles.th}>Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {backups.map((backup) => (
                    <tr key={backup.id}>
                      <td style={styles.td}>{backup.id}</td>
                      <td style={styles.td}>{backup.type}</td>
                      <td style={styles.td}>{formatDate(backup.createdAt)}</td>
                      <td style={styles.td}>{formatBytes(backup.sizeBytes)}</td>
                      <td style={styles.td}>
                        {backup.compressedSize
                          ? formatBytes(backup.compressedSize)
                          : '-'}
                      </td>
                      <td style={styles.td}>
                        <button
                          style={styles.smallButton}
                          onClick={() => handleRestoreBackup(backup.id)}
                        >
                          Restore
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        )}

        {/* Cache Tab */}
        {activeTab === 'cache' && stats && (
          <div style={styles.section}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
              <h2 style={styles.sectionTitle}>Cache Management</h2>
              <button style={styles.button} onClick={handleClearCache}>
                Clear All Cache
              </button>
            </div>

            <div style={styles.grid}>
              <div style={styles.card}>
                <h3 style={styles.cardTitle}>L1 Cache (Memory)</h3>
                <div style={styles.stat}>
                  <span>Hits:</span>
                  <span style={styles.statValue}>{stats.cache.l1Hits.toLocaleString()}</span>
                </div>
                <div style={styles.stat}>
                  <span>Misses:</span>
                  <span style={styles.statValue}>{stats.cache.l1Misses.toLocaleString()}</span>
                </div>
                <div style={styles.stat}>
                  <span>Entries:</span>
                  <span style={styles.statValue}>{stats.cache.l1Size}</span>
                </div>
              </div>

              <div style={styles.card}>
                <h3 style={styles.cardTitle}>L2 Cache (Disk)</h3>
                <div style={styles.stat}>
                  <span>Hits:</span>
                  <span style={styles.statValue}>{stats.cache.l2Hits.toLocaleString()}</span>
                </div>
                <div style={styles.stat}>
                  <span>Misses:</span>
                  <span style={styles.statValue}>{stats.cache.l2Misses.toLocaleString()}</span>
                </div>
                <div style={styles.stat}>
                  <span>Size:</span>
                  <span style={styles.statValue}>{formatBytes(stats.cache.l2Size)}</span>
                </div>
              </div>

              <div style={styles.card}>
                <h3 style={styles.cardTitle}>L3 Cache (Distributed)</h3>
                <div style={styles.stat}>
                  <span>Hits:</span>
                  <span style={styles.statValue}>{stats.cache.l3Hits.toLocaleString()}</span>
                </div>
                <div style={styles.stat}>
                  <span>Misses:</span>
                  <span style={styles.statValue}>{stats.cache.l3Misses.toLocaleString()}</span>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Health Tab */}
        {activeTab === 'health' && health && (
          <div style={styles.section}>
            <h2 style={styles.sectionTitle}>Health Status</h2>

            <div style={styles.card}>
              <div style={styles.stat}>
                <span>Overall Status:</span>
                <span style={{
                  ...styles.statValue,
                  color: health.healthy ? '#28a745' : '#dc3545',
                  fontWeight: 'bold',
                }}>
                  {health.healthy ? 'HEALTHY' : 'UNHEALTHY'}
                </span>
              </div>
              <div style={styles.stat}>
                <span>Response Time:</span>
                <span style={styles.statValue}>{health.responseTime} ms</span>
              </div>
              <div style={styles.stat}>
                <span>Timestamp:</span>
                <span style={styles.statValue}>{formatDate(health.timestamp)}</span>
              </div>
            </div>

            <h3 style={{ ...styles.sectionTitle, marginTop: '30px' }}>Components</h3>
            <div style={styles.grid}>
              {Object.entries(health.components).map(([component, status]) => (
                <div key={component} style={styles.card}>
                  <h4 style={styles.cardTitle}>{component.charAt(0).toUpperCase() + component.slice(1)}</h4>
                  <div style={styles.stat}>
                    <span>Status:</span>
                    <span style={{
                      ...styles.statValue,
                      color: status ? '#28a745' : '#dc3545',
                    }}>
                      {status ? 'OK' : 'ERROR'}
                    </span>
                  </div>
                </div>
              ))}
            </div>

            {health.errors && health.errors.length > 0 && (
              <>
                <h3 style={{ ...styles.sectionTitle, marginTop: '30px' }}>Errors</h3>
                <div style={styles.errorList}>
                  {health.errors.map((error, index) => (
                    <div key={index} style={styles.error}>
                      {error}
                    </div>
                  ))}
                </div>
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

// Styles
const styles: Record<string, React.CSSProperties> = {
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
