import React, { useState, useEffect, useCallback } from 'react';
export const ReportDataSource = ({ dataSources, selectedDataSource, onSelect, onSchemaExplore, onTestConnection, onCreateDataSource, readOnly = false, }) => {
    const [searchTerm, setSearchTerm] = useState('');
    const [selectedSource, setSelectedSource] = useState(selectedDataSource);
    const [schema, setSchema] = useState(null);
    const [loading, setLoading] = useState(false);
    const [testingConnection, setTestingConnection] = useState(false);
    const [connectionStatus, setConnectionStatus] = useState(null);
    const [expandedTables, setExpandedTables] = useState(new Set());
    const [selectedTable, setSelectedTable] = useState(null);
    const [viewMode, setViewMode] = useState('list');
    const filteredDataSources = dataSources.filter((ds) => ds.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        ds.type.toLowerCase().includes(searchTerm.toLowerCase()));
    useEffect(() => {
        if (selectedSource && onSchemaExplore && viewMode === 'schema') {
            loadSchema(selectedSource.id);
        }
    }, [selectedSource, viewMode]);
    const loadSchema = useCallback(async (dataSourceId) => {
        if (!onSchemaExplore)
            return;
        setLoading(true);
        try {
            const loadedSchema = await onSchemaExplore(dataSourceId);
            setSchema(loadedSchema);
        }
        catch (error) {
            console.error('Failed to load schema:', error);
            alert('Failed to load schema. Please try again.');
        }
        finally {
            setLoading(false);
        }
    }, [onSchemaExplore]);
    const handleTestConnection = useCallback(async () => {
        if (!selectedSource || !onTestConnection)
            return;
        setTestingConnection(true);
        setConnectionStatus(null);
        try {
            const success = await onTestConnection(selectedSource);
            setConnectionStatus(success ? 'success' : 'error');
        }
        catch (error) {
            console.error('Connection test failed:', error);
            setConnectionStatus('error');
        }
        finally {
            setTestingConnection(false);
        }
    }, [selectedSource, onTestConnection]);
    const handleSelectDataSource = useCallback((dataSource) => {
        setSelectedSource(dataSource);
        setSchema(dataSource.schema || null);
        setConnectionStatus(null);
        if (!readOnly) {
            onSelect(dataSource);
        }
    }, [readOnly, onSelect]);
    const toggleTableExpansion = useCallback((tableName) => {
        setExpandedTables((prev) => {
            const newSet = new Set(prev);
            if (newSet.has(tableName)) {
                newSet.delete(tableName);
            }
            else {
                newSet.add(tableName);
            }
            return newSet;
        });
    }, []);
    const renderDataSourceList = () => (React.createElement("div", { style: styles.dataSourceList },
        filteredDataSources.map((ds) => (React.createElement("div", { key: ds.id, onClick: () => handleSelectDataSource(ds), style: {
                ...styles.dataSourceItem,
                ...(selectedSource?.id === ds.id ? styles.dataSourceItemSelected : {}),
            } },
            React.createElement("div", { style: styles.dataSourceHeader },
                React.createElement("span", { style: styles.dataSourceIcon }, getDataSourceIcon(ds.type)),
                React.createElement("div", { style: styles.dataSourceInfo },
                    React.createElement("div", { style: styles.dataSourceName }, ds.name),
                    React.createElement("div", { style: styles.dataSourceType }, ds.type))),
            ds.metadata?.description && (React.createElement("div", { style: styles.dataSourceDescription }, ds.metadata.description)),
            ds.cacheConfig?.enabled && (React.createElement("div", { style: styles.cacheBadge },
                "\uD83D\uDDC4\uFE0F Cached (",
                ds.cacheConfig.ttl,
                "s)"))))),
        filteredDataSources.length === 0 && (React.createElement("div", { style: styles.emptyState },
            React.createElement("div", { style: styles.emptyStateIcon }, "\uD83D\uDD0D"),
            React.createElement("div", { style: styles.emptyStateText }, "No data sources found"),
            onCreateDataSource && (React.createElement("button", { onClick: onCreateDataSource, style: styles.createButton }, "+ Create Data Source"))))));
    const renderSchemaExplorer = () => {
        if (loading) {
            return (React.createElement("div", { style: styles.loadingContainer },
                React.createElement("div", { style: styles.spinner }, "Loading schema...")));
        }
        if (!schema || !schema.tables || schema.tables.length === 0) {
            return (React.createElement("div", { style: styles.emptyState },
                React.createElement("div", { style: styles.emptyStateIcon }, "\uD83D\uDCCA"),
                React.createElement("div", { style: styles.emptyStateText }, "No schema available"),
                onSchemaExplore && selectedSource && (React.createElement("button", { onClick: () => loadSchema(selectedSource.id), style: styles.refreshButton }, "\uD83D\uDD04 Load Schema"))));
        }
        return (React.createElement("div", { style: styles.schemaContainer },
            React.createElement("div", { style: styles.tablesPanel },
                React.createElement("div", { style: styles.tablesPanelHeader },
                    React.createElement("h4", { style: styles.tablesPanelTitle },
                        "Tables (",
                        schema.tables.length,
                        ")"),
                    React.createElement("input", { type: "text", placeholder: "Search tables...", style: styles.searchInput })),
                React.createElement("div", { style: styles.tablesList }, schema.tables.map((table) => (React.createElement("div", { key: table.name, style: styles.tableItem },
                    React.createElement("div", { onClick: () => toggleTableExpansion(table.name), style: styles.tableHeader },
                        React.createElement("span", { style: styles.expandIcon }, expandedTables.has(table.name) ? '▼' : '▶'),
                        React.createElement("span", { onClick: (e) => {
                                e.stopPropagation();
                                setSelectedTable(table.name);
                            }, style: {
                                ...styles.tableName,
                                ...(selectedTable === table.name ? styles.tableNameSelected : {}),
                            } }, table.displayName || table.name),
                        React.createElement("span", { style: styles.fieldCount },
                            table.fields.length,
                            " fields")),
                    expandedTables.has(table.name) && (React.createElement("div", { style: styles.fieldsList }, table.fields.map((field) => (React.createElement("div", { key: field.name, style: styles.fieldItem, title: field.description },
                        React.createElement("span", { style: styles.fieldIcon }, getFieldIcon(field.dataType)),
                        React.createElement("span", { style: styles.fieldName }, field.displayName || field.name),
                        React.createElement("span", { style: styles.fieldType }, field.dataType),
                        field.name === table.primaryKey && (React.createElement("span", { style: styles.primaryKeyBadge }, "PK")))))))))))),
            selectedTable && (React.createElement("div", { style: styles.tableDetailsPanel }, renderTableDetails(schema.tables.find((t) => t.name === selectedTable))))));
    };
    const renderTableDetails = (table) => (React.createElement("div", null,
        React.createElement("h3", { style: styles.tableDetailsTitle }, table.displayName || table.name),
        table.description && (React.createElement("p", { style: styles.tableDetailsDescription }, table.description)),
        React.createElement("div", { style: styles.tableDetailsSection },
            React.createElement("h4", { style: styles.sectionTitle }, "Fields"),
            React.createElement("table", { style: styles.detailsTable },
                React.createElement("thead", null,
                    React.createElement("tr", null,
                        React.createElement("th", { style: styles.detailsTableHeader }, "Name"),
                        React.createElement("th", { style: styles.detailsTableHeader }, "Type"),
                        React.createElement("th", { style: styles.detailsTableHeader }, "Nullable"),
                        React.createElement("th", { style: styles.detailsTableHeader }, "Aggregation"))),
                React.createElement("tbody", null, table.fields.map((field) => (React.createElement("tr", { key: field.name },
                    React.createElement("td", { style: styles.detailsTableCell },
                        field.displayName || field.name,
                        field.name === table.primaryKey && (React.createElement("span", { style: styles.primaryKeyBadge }, "PK"))),
                    React.createElement("td", { style: styles.detailsTableCell }, field.dataType),
                    React.createElement("td", { style: styles.detailsTableCell }, field.nullable ? 'Yes' : 'No'),
                    React.createElement("td", { style: styles.detailsTableCell }, field.defaultAggregation || '-'))))))),
        schema?.relationships && schema.relationships.length > 0 && (React.createElement("div", { style: styles.tableDetailsSection },
            React.createElement("h4", { style: styles.sectionTitle }, "Relationships"),
            React.createElement("div", { style: styles.relationshipsList }, schema.relationships
                .filter((rel) => rel.sourceTable === table.name || rel.targetTable === table.name)
                .map((rel, index) => (React.createElement("div", { key: index, style: styles.relationshipItem },
                React.createElement("div", { style: styles.relationshipName }, rel.name),
                React.createElement("div", { style: styles.relationshipDetails },
                    rel.sourceTable,
                    ".",
                    rel.sourceField,
                    " \u2192 ",
                    rel.targetTable,
                    ".",
                    rel.targetField),
                React.createElement("div", { style: styles.relationshipType }, rel.type)))))))));
    return (React.createElement("div", { style: styles.container },
        React.createElement("div", { style: styles.header },
            React.createElement("div", { style: styles.headerLeft },
                React.createElement("h3", { style: styles.title }, "Data Source"),
                selectedSource && (React.createElement("span", { style: styles.selectedBadge },
                    getDataSourceIcon(selectedSource.type),
                    " ",
                    selectedSource.name))),
            React.createElement("div", { style: styles.headerRight }, selectedSource && (React.createElement(React.Fragment, null,
                React.createElement("div", { style: styles.viewModeToggle },
                    React.createElement("button", { onClick: () => setViewMode('list'), style: {
                            ...styles.viewModeButton,
                            ...(viewMode === 'list' ? styles.viewModeButtonActive : {}),
                        } }, "\uD83D\uDCCB List"),
                    React.createElement("button", { onClick: () => setViewMode('schema'), style: {
                            ...styles.viewModeButton,
                            ...(viewMode === 'schema' ? styles.viewModeButtonActive : {}),
                        } }, "\uD83D\uDCCA Schema")),
                onTestConnection && (React.createElement("button", { onClick: handleTestConnection, disabled: testingConnection, style: styles.testButton }, testingConnection ? '⟳ Testing...' : '🔌 Test Connection')))))),
        connectionStatus && (React.createElement("div", { style: {
                ...styles.statusBanner,
                ...(connectionStatus === 'success'
                    ? styles.statusBannerSuccess
                    : styles.statusBannerError),
            } }, connectionStatus === 'success' ? '✓ Connection successful' : '✗ Connection failed')),
        viewMode === 'list' && (React.createElement("div", { style: styles.searchContainer },
            React.createElement("input", { type: "text", placeholder: "Search data sources...", value: searchTerm, onChange: (e) => setSearchTerm(e.target.value), style: styles.searchInput }),
            onCreateDataSource && (React.createElement("button", { onClick: onCreateDataSource, style: styles.createButton }, "+ New Data Source")))),
        React.createElement("div", { style: styles.content }, viewMode === 'list' ? renderDataSourceList() : renderSchemaExplorer())));
};
function getDataSourceIcon(type) {
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
function getFieldIcon(dataType) {
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
const styles = {
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
//# sourceMappingURL=ReportDataSource.js.map