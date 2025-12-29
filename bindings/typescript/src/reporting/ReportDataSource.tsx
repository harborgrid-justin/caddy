/**
 * CADDY v0.4.0 - Report Data Source Component
 * $650M Platform - Production Ready
 *
 * Comprehensive data source selector and configuration with schema
 * exploration, connection testing, and query building capabilities.
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  DataSource,
  DataSourceSchema,
  Table,
  Field,
  Relationship,
} from './types';

export interface ReportDataSourceProps {
  dataSources: DataSource[];
  selectedDataSource?: DataSource;
  onSelect: (dataSource: DataSource) => void;
  onSchemaExplore?: (dataSourceId: string) => Promise<DataSourceSchema>;
  onTestConnection?: (dataSource: DataSource) => Promise<boolean>;
  onCreateDataSource?: () => void;
  readOnly?: boolean;
}

export const ReportDataSource: React.FC<ReportDataSourceProps> = ({
  dataSources,
  selectedDataSource,
  onSelect,
  onSchemaExplore,
  onTestConnection,
  onCreateDataSource,
  readOnly = false,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedSource, setSelectedSource] = useState<DataSource | undefined>(selectedDataSource);
  const [schema, setSchema] = useState<DataSourceSchema | null>(null);
  const [loading, setLoading] = useState(false);
  const [testingConnection, setTestingConnection] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'success' | 'error' | null>(null);
  const [expandedTables, setExpandedTables] = useState<Set<string>>(new Set());
  const [selectedTable, setSelectedTable] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'list' | 'schema'>('list');

  // Filter data sources by search term
  const filteredDataSources = dataSources.filter((ds) =>
    ds.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    ds.type.toLowerCase().includes(searchTerm.toLowerCase())
  );

  // Load schema when data source is selected
  useEffect(() => {
    if (selectedSource && onSchemaExplore && viewMode === 'schema') {
      loadSchema(selectedSource.id);
    }
  }, [selectedSource, viewMode]);

  // Load schema
  const loadSchema = useCallback(
    async (dataSourceId: string) => {
      if (!onSchemaExplore) return;

      setLoading(true);
      try {
        const loadedSchema = await onSchemaExplore(dataSourceId);
        setSchema(loadedSchema);
      } catch (error) {
        console.error('Failed to load schema:', error);
        alert('Failed to load schema. Please try again.');
      } finally {
        setLoading(false);
      }
    },
    [onSchemaExplore]
  );

  // Test connection
  const handleTestConnection = useCallback(async () => {
    if (!selectedSource || !onTestConnection) return;

    setTestingConnection(true);
    setConnectionStatus(null);

    try {
      const success = await onTestConnection(selectedSource);
      setConnectionStatus(success ? 'success' : 'error');
    } catch (error) {
      console.error('Connection test failed:', error);
      setConnectionStatus('error');
    } finally {
      setTestingConnection(false);
    }
  }, [selectedSource, onTestConnection]);

  // Select data source
  const handleSelectDataSource = useCallback(
    (dataSource: DataSource) => {
      setSelectedSource(dataSource);
      setSchema(dataSource.schema || null);
      setConnectionStatus(null);
      if (!readOnly) {
        onSelect(dataSource);
      }
    },
    [readOnly, onSelect]
  );

  // Toggle table expansion
  const toggleTableExpansion = useCallback((tableName: string) => {
    setExpandedTables((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(tableName)) {
        newSet.delete(tableName);
      } else {
        newSet.add(tableName);
      }
      return newSet;
    });
  }, []);

  // Render data source list
  const renderDataSourceList = () => (
    <div style={styles.dataSourceList}>
      {filteredDataSources.map((ds) => (
        <div
          key={ds.id}
          onClick={() => handleSelectDataSource(ds)}
          style={{
            ...styles.dataSourceItem,
            ...(selectedSource?.id === ds.id ? styles.dataSourceItemSelected : {}),
          }}
        >
          <div style={styles.dataSourceHeader}>
            <span style={styles.dataSourceIcon}>{getDataSourceIcon(ds.type)}</span>
            <div style={styles.dataSourceInfo}>
              <div style={styles.dataSourceName}>{ds.name}</div>
              <div style={styles.dataSourceType}>{ds.type}</div>
            </div>
          </div>
          {ds.metadata?.description && (
            <div style={styles.dataSourceDescription}>{ds.metadata.description}</div>
          )}
          {ds.cacheConfig?.enabled && (
            <div style={styles.cacheBadge}>
              🗄️ Cached ({ds.cacheConfig.ttl}s)
            </div>
          )}
        </div>
      ))}
      {filteredDataSources.length === 0 && (
        <div style={styles.emptyState}>
          <div style={styles.emptyStateIcon}>🔍</div>
          <div style={styles.emptyStateText}>No data sources found</div>
          {onCreateDataSource && (
            <button onClick={onCreateDataSource} style={styles.createButton}>
              + Create Data Source
            </button>
          )}
        </div>
      )}
    </div>
  );

  // Render schema explorer
  const renderSchemaExplorer = () => {
    if (loading) {
      return (
        <div style={styles.loadingContainer}>
          <div style={styles.spinner}>Loading schema...</div>
        </div>
      );
    }

    if (!schema || !schema.tables || schema.tables.length === 0) {
      return (
        <div style={styles.emptyState}>
          <div style={styles.emptyStateIcon}>📊</div>
          <div style={styles.emptyStateText}>No schema available</div>
          {onSchemaExplore && selectedSource && (
            <button
              onClick={() => loadSchema(selectedSource.id)}
              style={styles.refreshButton}
            >
              🔄 Load Schema
            </button>
          )}
        </div>
      );
    }

    return (
      <div style={styles.schemaContainer}>
        <div style={styles.tablesPanel}>
          <div style={styles.tablesPanelHeader}>
            <h4 style={styles.tablesPanelTitle}>Tables ({schema.tables.length})</h4>
            <input
              type="text"
              placeholder="Search tables..."
              style={styles.searchInput}
            />
          </div>
          <div style={styles.tablesList}>
            {schema.tables.map((table) => (
              <div key={table.name} style={styles.tableItem}>
                <div
                  onClick={() => toggleTableExpansion(table.name)}
                  style={styles.tableHeader}
                >
                  <span style={styles.expandIcon}>
                    {expandedTables.has(table.name) ? '▼' : '▶'}
                  </span>
                  <span
                    onClick={(e) => {
                      e.stopPropagation();
                      setSelectedTable(table.name);
                    }}
                    style={{
                      ...styles.tableName,
                      ...(selectedTable === table.name ? styles.tableNameSelected : {}),
                    }}
                  >
                    {table.displayName || table.name}
                  </span>
                  <span style={styles.fieldCount}>{table.fields.length} fields</span>
                </div>
                {expandedTables.has(table.name) && (
                  <div style={styles.fieldsList}>
                    {table.fields.map((field) => (
                      <div
                        key={field.name}
                        style={styles.fieldItem}
                        title={field.description}
                      >
                        <span style={styles.fieldIcon}>{getFieldIcon(field.dataType)}</span>
                        <span style={styles.fieldName}>{field.displayName || field.name}</span>
                        <span style={styles.fieldType}>{field.dataType}</span>
                        {field.name === table.primaryKey && (
                          <span style={styles.primaryKeyBadge}>PK</span>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>

        {selectedTable && (
          <div style={styles.tableDetailsPanel}>
            {renderTableDetails(schema.tables.find((t) => t.name === selectedTable)!)}
          </div>
        )}
      </div>
    );
  };

  // Render table details
  const renderTableDetails = (table: Table) => (
    <div>
      <h3 style={styles.tableDetailsTitle}>{table.displayName || table.name}</h3>
      {table.description && (
        <p style={styles.tableDetailsDescription}>{table.description}</p>
      )}

      <div style={styles.tableDetailsSection}>
        <h4 style={styles.sectionTitle}>Fields</h4>
        <table style={styles.detailsTable}>
          <thead>
            <tr>
              <th style={styles.detailsTableHeader}>Name</th>
              <th style={styles.detailsTableHeader}>Type</th>
              <th style={styles.detailsTableHeader}>Nullable</th>
              <th style={styles.detailsTableHeader}>Aggregation</th>
            </tr>
          </thead>
          <tbody>
            {table.fields.map((field) => (
              <tr key={field.name}>
                <td style={styles.detailsTableCell}>
                  {field.displayName || field.name}
                  {field.name === table.primaryKey && (
                    <span style={styles.primaryKeyBadge}>PK</span>
                  )}
                </td>
                <td style={styles.detailsTableCell}>{field.dataType}</td>
                <td style={styles.detailsTableCell}>{field.nullable ? 'Yes' : 'No'}</td>
                <td style={styles.detailsTableCell}>
                  {field.defaultAggregation || '-'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {schema?.relationships && schema.relationships.length > 0 && (
        <div style={styles.tableDetailsSection}>
          <h4 style={styles.sectionTitle}>Relationships</h4>
          <div style={styles.relationshipsList}>
            {schema.relationships
              .filter(
                (rel) => rel.sourceTable === table.name || rel.targetTable === table.name
              )
              .map((rel, index) => (
                <div key={index} style={styles.relationshipItem}>
                  <div style={styles.relationshipName}>{rel.name}</div>
                  <div style={styles.relationshipDetails}>
                    {rel.sourceTable}.{rel.sourceField} → {rel.targetTable}.{rel.targetField}
                  </div>
                  <div style={styles.relationshipType}>{rel.type}</div>
                </div>
              ))}
          </div>
        </div>
      )}
    </div>
  );

  return (
    <div style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <div style={styles.headerLeft}>
          <h3 style={styles.title}>Data Source</h3>
          {selectedSource && (
            <span style={styles.selectedBadge}>
              {getDataSourceIcon(selectedSource.type)} {selectedSource.name}
            </span>
          )}
        </div>
        <div style={styles.headerRight}>
          {selectedSource && (
            <>
              <div style={styles.viewModeToggle}>
                <button
                  onClick={() => setViewMode('list')}
                  style={{
                    ...styles.viewModeButton,
                    ...(viewMode === 'list' ? styles.viewModeButtonActive : {}),
                  }}
                >
                  📋 List
                </button>
                <button
                  onClick={() => setViewMode('schema')}
                  style={{
                    ...styles.viewModeButton,
                    ...(viewMode === 'schema' ? styles.viewModeButtonActive : {}),
                  }}
                >
                  📊 Schema
                </button>
              </div>
              {onTestConnection && (
                <button
                  onClick={handleTestConnection}
                  disabled={testingConnection}
                  style={styles.testButton}
                >
                  {testingConnection ? '⟳ Testing...' : '🔌 Test Connection'}
                </button>
              )}
            </>
          )}
        </div>
      </div>

      {/* Connection Status */}
      {connectionStatus && (
        <div
          style={{
            ...styles.statusBanner,
            ...(connectionStatus === 'success'
              ? styles.statusBannerSuccess
              : styles.statusBannerError),
          }}
        >
          {connectionStatus === 'success' ? '✓ Connection successful' : '✗ Connection failed'}
        </div>
      )}

      {/* Search */}
      {viewMode === 'list' && (
        <div style={styles.searchContainer}>
          <input
            type="text"
            placeholder="Search data sources..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            style={styles.searchInput}
          />
          {onCreateDataSource && (
            <button onClick={onCreateDataSource} style={styles.createButton}>
              + New Data Source
            </button>
          )}
        </div>
      )}

      {/* Content */}
      <div style={styles.content}>
        {viewMode === 'list' ? renderDataSourceList() : renderSchemaExplorer()}
      </div>
    </div>
  );
};

// Helper functions
function getDataSourceIcon(type: string): string {
  switch (type) {
    case 'database':
      return '🗄️';
    case 'api':
      return '🔌';
    case 'file':
      return '📁';
    case 'custom':
      return '⚙️';
    default:
      return '📊';
  }
}

function getFieldIcon(dataType: string): string {
  switch (dataType) {
    case 'string':
      return '📝';
    case 'number':
      return '🔢';
    case 'date':
      return '📅';
    case 'boolean':
      return '☑️';
    case 'json':
      return '{}';
    case 'array':
      return '[]';
    default:
      return '•';
  }
}

// Styles
const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#ffffff',
    fontFamily: 'Inter, system-ui, sans-serif',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '16px',
    borderBottom: '1px solid #e2e8f0',
  },
  headerLeft: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
  },
  headerRight: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  title: {
    fontSize: '16px',
    fontWeight: 600,
    margin: 0,
    color: '#1e293b',
  },
  selectedBadge: {
    fontSize: '13px',
    padding: '4px 12px',
    backgroundColor: '#e0e7ff',
    color: '#3730a3',
    borderRadius: '12px',
    fontWeight: 500,
  },
  viewModeToggle: {
    display: 'flex',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    overflow: 'hidden',
  },
  viewModeButton: {
    padding: '6px 12px',
    border: 'none',
    backgroundColor: '#ffffff',
    cursor: 'pointer',
    fontSize: '13px',
    fontWeight: 500,
  },
  viewModeButtonActive: {
    backgroundColor: '#2563eb',
    color: '#ffffff',
  },
  testButton: {
    padding: '6px 12px',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    backgroundColor: '#ffffff',
    cursor: 'pointer',
    fontSize: '13px',
    fontWeight: 500,
  },
  statusBanner: {
    padding: '8px 16px',
    fontSize: '13px',
    fontWeight: 500,
  },
  statusBannerSuccess: {
    backgroundColor: '#d1fae5',
    color: '#065f46',
  },
  statusBannerError: {
    backgroundColor: '#fee2e2',
    color: '#991b1b',
  },
  searchContainer: {
    display: 'flex',
    gap: '8px',
    padding: '16px',
    borderBottom: '1px solid #e2e8f0',
  },
  searchInput: {
    flex: 1,
    padding: '8px 12px',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    fontSize: '14px',
  },
  createButton: {
    padding: '8px 16px',
    border: 'none',
    borderRadius: '6px',
    backgroundColor: '#2563eb',
    color: '#ffffff',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer',
  },
  content: {
    flex: 1,
    overflow: 'auto',
  },
  dataSourceList: {
    padding: '16px',
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
  },
  dataSourceItem: {
    padding: '12px',
    border: '1px solid #e2e8f0',
    borderRadius: '8px',
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  dataSourceItemSelected: {
    borderColor: '#2563eb',
    backgroundColor: '#eff6ff',
  },
  dataSourceHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
  },
  dataSourceIcon: {
    fontSize: '24px',
  },
  dataSourceInfo: {
    flex: 1,
  },
  dataSourceName: {
    fontSize: '14px',
    fontWeight: 600,
    color: '#1e293b',
  },
  dataSourceType: {
    fontSize: '12px',
    color: '#64748b',
    textTransform: 'capitalize',
  },
  dataSourceDescription: {
    fontSize: '12px',
    color: '#64748b',
    marginTop: '8px',
  },
  cacheBadge: {
    fontSize: '11px',
    padding: '2px 6px',
    backgroundColor: '#dbeafe',
    color: '#1e40af',
    borderRadius: '4px',
    display: 'inline-block',
    marginTop: '8px',
  },
  emptyState: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '48px',
    gap: '16px',
  },
  emptyStateIcon: {
    fontSize: '48px',
  },
  emptyStateText: {
    fontSize: '14px',
    color: '#64748b',
  },
  refreshButton: {
    padding: '8px 16px',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    backgroundColor: '#ffffff',
    cursor: 'pointer',
    fontSize: '13px',
  },
  loadingContainer: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '48px',
  },
  spinner: {
    fontSize: '14px',
    color: '#64748b',
  },
  schemaContainer: {
    display: 'flex',
    height: '100%',
  },
  tablesPanel: {
    width: '300px',
    borderRight: '1px solid #e2e8f0',
    display: 'flex',
    flexDirection: 'column',
  },
  tablesPanelHeader: {
    padding: '12px',
    borderBottom: '1px solid #e2e8f0',
  },
  tablesPanelTitle: {
    fontSize: '14px',
    fontWeight: 600,
    margin: '0 0 8px 0',
    color: '#1e293b',
  },
  tablesList: {
    flex: 1,
    overflow: 'auto',
  },
  tableItem: {
    borderBottom: '1px solid #e2e8f0',
  },
  tableHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '8px 12px',
    cursor: 'pointer',
    transition: 'background-color 0.2s',
  },
  expandIcon: {
    fontSize: '10px',
    color: '#64748b',
  },
  tableName: {
    flex: 1,
    fontSize: '13px',
    fontWeight: 500,
    color: '#1e293b',
  },
  tableNameSelected: {
    color: '#2563eb',
  },
  fieldCount: {
    fontSize: '11px',
    color: '#94a3b8',
  },
  fieldsList: {
    backgroundColor: '#f8fafc',
    padding: '4px 0',
  },
  fieldItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '6px 12px 6px 32px',
    fontSize: '12px',
  },
  fieldIcon: {
    fontSize: '14px',
  },
  fieldName: {
    flex: 1,
    color: '#475569',
  },
  fieldType: {
    fontSize: '11px',
    color: '#94a3b8',
  },
  primaryKeyBadge: {
    fontSize: '10px',
    padding: '1px 4px',
    backgroundColor: '#fef3c7',
    color: '#92400e',
    borderRadius: '3px',
    fontWeight: 600,
    marginLeft: '4px',
  },
  tableDetailsPanel: {
    flex: 1,
    padding: '16px',
    overflow: 'auto',
  },
  tableDetailsTitle: {
    fontSize: '18px',
    fontWeight: 600,
    margin: '0 0 8px 0',
    color: '#1e293b',
  },
  tableDetailsDescription: {
    fontSize: '13px',
    color: '#64748b',
    marginBottom: '16px',
  },
  tableDetailsSection: {
    marginBottom: '24px',
  },
  sectionTitle: {
    fontSize: '14px',
    fontWeight: 600,
    margin: '0 0 12px 0',
    color: '#1e293b',
  },
  detailsTable: {
    width: '100%',
    borderCollapse: 'collapse',
    fontSize: '13px',
  },
  detailsTableHeader: {
    padding: '8px',
    textAlign: 'left',
    borderBottom: '2px solid #e2e8f0',
    backgroundColor: '#f8fafc',
    fontWeight: 600,
    color: '#475569',
  },
  detailsTableCell: {
    padding: '8px',
    borderBottom: '1px solid #e2e8f0',
    color: '#475569',
  },
  relationshipsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
  },
  relationshipItem: {
    padding: '8px',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    backgroundColor: '#f8fafc',
  },
  relationshipName: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#1e293b',
  },
  relationshipDetails: {
    fontSize: '11px',
    color: '#64748b',
    marginTop: '4px',
    fontFamily: 'monospace',
  },
  relationshipType: {
    fontSize: '10px',
    color: '#94a3b8',
    marginTop: '4px',
  },
};

export default ReportDataSource;
