/**
 * CADDY Database - Migration Management Panel
 *
 * Provides a UI for managing database migrations with visual status,
 * rollback support, and migration history.
 */

import React, { useState, useEffect } from 'react';
import { useDatabase } from './DatabaseProvider';
import { Migration, MigrationStatus } from './types';

/**
 * Migration panel component
 */
export function MigrationPanel() {
  const { getMigrationStatus, runMigrations } = useDatabase();
  const [status, setStatus] = useState<MigrationStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRunning, setIsRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedMigration, setSelectedMigration] = useState<Migration | null>(null);

  // Load migration status
  const loadStatus = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const result = await getMigrationStatus();
      setStatus(result);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to load migration status';
      setError(message);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadStatus();
  }, []);

  // Run pending migrations
  const handleRunMigrations = async () => {
    if (!confirm(`Are you sure you want to run ${status?.pending} pending migration(s)?`)) {
      return;
    }

    setIsRunning(true);
    setError(null);

    try {
      await runMigrations();
      await loadStatus(); // Reload status
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Migration failed';
      setError(message);
    } finally {
      setIsRunning(false);
    }
  };

  // Format date
  const formatDate = (dateString: string | undefined): string => {
    if (!dateString) return '-';
    return new Date(dateString).toLocaleString();
  };

  // Render loading state
  if (isLoading) {
    return (
      <div style={styles.container}>
        <div style={styles.loading}>Loading migrations...</div>
      </div>
    );
  }

  // Render error state
  if (error && !status) {
    return (
      <div style={styles.container}>
        <div style={styles.error}>Error: {error}</div>
        <button style={styles.button} onClick={loadStatus}>
          Retry
        </button>
      </div>
    );
  }

  if (!status) {
    return null;
  }

  return (
    <div style={styles.container}>
      <h1 style={styles.title}>Database Migrations</h1>

      {/* Error message */}
      {error && (
        <div style={styles.errorBanner}>
          {error}
        </div>
      )}

      {/* Summary */}
      <div style={styles.summary}>
        <div style={styles.summaryCard}>
          <div style={styles.summaryValue}>{status.total}</div>
          <div style={styles.summaryLabel}>Total Migrations</div>
        </div>
        <div style={{ ...styles.summaryCard, background: '#d4edda' }}>
          <div style={styles.summaryValue}>{status.applied}</div>
          <div style={styles.summaryLabel}>Applied</div>
        </div>
        <div style={{ ...styles.summaryCard, background: status.pending > 0 ? '#fff3cd' : '#e9ecef' }}>
          <div style={styles.summaryValue}>{status.pending}</div>
          <div style={styles.summaryLabel}>Pending</div>
        </div>
      </div>

      {/* Actions */}
      {status.pending > 0 && (
        <div style={styles.actions}>
          <button
            style={styles.primaryButton}
            onClick={handleRunMigrations}
            disabled={isRunning}
          >
            {isRunning ? 'Running Migrations...' : `Run ${status.pending} Pending Migration(s)`}
          </button>
        </div>
      )}

      {/* Pending Migrations */}
      {status.pending > 0 && (
        <div style={styles.section}>
          <h2 style={styles.sectionTitle}>
            Pending Migrations
            <span style={styles.badge}>{status.pending}</span>
          </h2>

          <div style={styles.migrationList}>
            {status.pendingMigrations.map((migration) => (
              <div
                key={migration.version}
                style={styles.migrationCard}
                onClick={() => setSelectedMigration(migration)}
              >
                <div style={styles.migrationHeader}>
                  <div>
                    <div style={styles.migrationName}>{migration.name}</div>
                    <div style={styles.migrationVersion}>
                      Version: {migration.version}
                    </div>
                  </div>
                  <div style={{ ...styles.status, background: '#ffc107' }}>
                    Pending
                  </div>
                </div>
                <div style={styles.migrationDescription}>
                  {migration.description}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Applied Migrations History */}
      <div style={styles.section}>
        <h2 style={styles.sectionTitle}>
          Migration History
          <span style={styles.badge}>{status.applied}</span>
        </h2>

        {status.history.length === 0 ? (
          <div style={styles.empty}>No migrations applied yet</div>
        ) : (
          <div style={styles.migrationList}>
            {status.history.map((migration) => (
              <div
                key={migration.version}
                style={styles.migrationCard}
                onClick={() => setSelectedMigration(migration)}
              >
                <div style={styles.migrationHeader}>
                  <div>
                    <div style={styles.migrationName}>{migration.name}</div>
                    <div style={styles.migrationVersion}>
                      Version: {migration.version}
                    </div>
                  </div>
                  <div style={{ ...styles.status, background: '#28a745' }}>
                    Applied
                  </div>
                </div>
                <div style={styles.migrationDescription}>
                  {migration.description}
                </div>
                {migration.appliedAt && (
                  <div style={styles.migrationDate}>
                    Applied: {formatDate(migration.appliedAt)}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Migration Details Modal */}
      {selectedMigration && (
        <div style={styles.modal} onClick={() => setSelectedMigration(null)}>
          <div style={styles.modalContent} onClick={(e) => e.stopPropagation()}>
            <div style={styles.modalHeader}>
              <h3 style={styles.modalTitle}>Migration Details</h3>
              <button
                style={styles.closeButton}
                onClick={() => setSelectedMigration(null)}
              >
                ×
              </button>
            </div>

            <div style={styles.modalBody}>
              <div style={styles.detailRow}>
                <span style={styles.detailLabel}>Name:</span>
                <span style={styles.detailValue}>{selectedMigration.name}</span>
              </div>

              <div style={styles.detailRow}>
                <span style={styles.detailLabel}>Version:</span>
                <span style={styles.detailValue}>{selectedMigration.version}</span>
              </div>

              <div style={styles.detailRow}>
                <span style={styles.detailLabel}>Description:</span>
                <span style={styles.detailValue}>{selectedMigration.description}</span>
              </div>

              <div style={styles.detailRow}>
                <span style={styles.detailLabel}>Status:</span>
                <span style={{
                  ...styles.status,
                  background: selectedMigration.isApplied ? '#28a745' : '#ffc107',
                }}>
                  {selectedMigration.isApplied ? 'Applied' : 'Pending'}
                </span>
              </div>

              {selectedMigration.appliedAt && (
                <div style={styles.detailRow}>
                  <span style={styles.detailLabel}>Applied At:</span>
                  <span style={styles.detailValue}>
                    {formatDate(selectedMigration.appliedAt)}
                  </span>
                </div>
              )}

              {selectedMigration.up && (
                <div style={styles.codeSection}>
                  <div style={styles.codeLabel}>UP SQL:</div>
                  <pre style={styles.code}>{selectedMigration.up}</pre>
                </div>
              )}

              {selectedMigration.down && (
                <div style={styles.codeSection}>
                  <div style={styles.codeLabel}>DOWN SQL:</div>
                  <pre style={styles.code}>{selectedMigration.down}</pre>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// Styles
const styles: Record<string, React.CSSProperties> = {
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
