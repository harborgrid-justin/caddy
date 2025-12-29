/**
 * CADDY v0.4.0 - Report Fields Component
 * $650M Platform - Production Ready
 *
 * Advanced field picker with drag-and-drop, aggregations, calculations,
 * and formula builder for complex report queries.
 */

import React, { useState, useCallback } from 'react';
import {
  Field,
  SelectField,
  AggregationType,
  Table,
  DataType,
} from './types';

export interface ReportFieldsProps {
  availableTables: Table[];
  selectedFields: SelectField[];
  onChange: (fields: SelectField[]) => void;
  readOnly?: boolean;
  showAggregations?: boolean;
  showCalculations?: boolean;
}

export const ReportFields: React.FC<ReportFieldsProps> = ({
  availableTables,
  selectedFields,
  onChange,
  readOnly = false,
  showAggregations = true,
  showCalculations = true,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedTables, setExpandedTables] = useState<Set<string>>(new Set());
  const [editingField, setEditingField] = useState<string | null>(null);
  const [showCalculationBuilder, setShowCalculationBuilder] = useState(false);

  // Toggle table expansion
  const toggleTable = useCallback((tableName: string) => {
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

  // Add field to selection
  const addField = useCallback(
    (table: Table, field: Field) => {
      if (readOnly) return;

      const newField: SelectField = {
        field: `${table.name}.${field.name}`,
        alias: field.displayName || field.name,
        aggregation: field.defaultAggregation,
        format: field.format,
      };

      onChange([...selectedFields, newField]);
    },
    [selectedFields, onChange, readOnly]
  );

  // Remove field from selection
  const removeField = useCallback(
    (index: number) => {
      if (readOnly) return;
      onChange(selectedFields.filter((_, i) => i !== index));
    },
    [selectedFields, onChange, readOnly]
  );

  // Update field properties
  const updateField = useCallback(
    (index: number, updates: Partial<SelectField>) => {
      if (readOnly) return;
      onChange(
        selectedFields.map((field, i) => (i === index ? { ...field, ...updates } : field))
      );
    },
    [selectedFields, onChange, readOnly]
  );

  // Move field up/down
  const moveField = useCallback(
    (index: number, direction: 'up' | 'down') => {
      if (readOnly) return;
      const newFields = [...selectedFields];
      const targetIndex = direction === 'up' ? index - 1 : index + 1;

      if (targetIndex < 0 || targetIndex >= newFields.length) return;

      [newFields[index], newFields[targetIndex]] = [newFields[targetIndex], newFields[index]];
      onChange(newFields);
    },
    [selectedFields, onChange, readOnly]
  );

  // Add calculated field
  const addCalculatedField = useCallback(
    (calculation: { expression: string; fields: string[]; alias: string }) => {
      if (readOnly) return;

      const newField: SelectField = {
        field: 'calculated',
        alias: calculation.alias,
        calculation: {
          expression: calculation.expression,
          fields: calculation.fields,
        },
      };

      onChange([...selectedFields, newField]);
      setShowCalculationBuilder(false);
    },
    [selectedFields, onChange, readOnly]
  );

  // Filter available fields
  const filteredTables = availableTables
    .map((table) => ({
      ...table,
      fields: table.fields.filter(
        (field) =>
          field.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
          (field.displayName?.toLowerCase() || '').includes(searchTerm.toLowerCase())
      ),
    }))
    .filter((table) => table.fields.length > 0);

  // Check if field is selected
  const isFieldSelected = useCallback(
    (tableName: string, fieldName: string) => {
      return selectedFields.some((sf) => sf.field === `${tableName}.${fieldName}`);
    },
    [selectedFields]
  );

  return (
    <div style={styles.container}>
      {/* Available Fields Panel */}
      <div style={styles.availablePanel}>
        <div style={styles.panelHeader}>
          <h3 style={styles.panelTitle}>Available Fields</h3>
          <input
            type="text"
            placeholder="Search fields..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            style={styles.searchInput}
          />
        </div>

        <div style={styles.fieldsList}>
          {filteredTables.map((table) => (
            <div key={table.name} style={styles.tableGroup}>
              <div onClick={() => toggleTable(table.name)} style={styles.tableGroupHeader}>
                <span style={styles.expandIcon}>
                  {expandedTables.has(table.name) ? '▼' : '▶'}
                </span>
                <span style={styles.tableGroupName}>
                  {table.displayName || table.name}
                </span>
                <span style={styles.fieldCount}>({table.fields.length})</span>
              </div>

              {expandedTables.has(table.name) && (
                <div style={styles.tableFields}>
                  {table.fields.map((field) => {
                    const selected = isFieldSelected(table.name, field.name);
                    return (
                      <div
                        key={field.name}
                        onClick={() => !selected && addField(table, field)}
                        style={{
                          ...styles.fieldItem,
                          ...(selected ? styles.fieldItemSelected : {}),
                          cursor: selected || readOnly ? 'default' : 'pointer',
                        }}
                        draggable={!readOnly}
                        title={field.description}
                      >
                        <span style={styles.fieldIcon}>{getFieldTypeIcon(field.dataType)}</span>
                        <span style={styles.fieldName}>
                          {field.displayName || field.name}
                        </span>
                        <span style={styles.fieldType}>{field.dataType}</span>
                        {selected && <span style={styles.selectedBadge}>✓</span>}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          ))}

          {filteredTables.length === 0 && (
            <div style={styles.emptyState}>
              <div style={styles.emptyStateIcon}>🔍</div>
              <div style={styles.emptyStateText}>No fields found</div>
            </div>
          )}
        </div>

        {showCalculations && !readOnly && (
          <div style={styles.panelFooter}>
            <button
              onClick={() => setShowCalculationBuilder(true)}
              style={styles.addCalculationButton}
            >
              ƒ Add Calculated Field
            </button>
          </div>
        )}
      </div>

      {/* Selected Fields Panel */}
      <div style={styles.selectedPanel}>
        <div style={styles.panelHeader}>
          <h3 style={styles.panelTitle}>
            Selected Fields ({selectedFields.length})
          </h3>
          {!readOnly && selectedFields.length > 0 && (
            <button
              onClick={() => onChange([])}
              style={styles.clearButton}
            >
              Clear All
            </button>
          )}
        </div>

        <div style={styles.selectedList}>
          {selectedFields.map((field, index) => (
            <div
              key={index}
              style={{
                ...styles.selectedField,
                ...(editingField === field.field ? styles.selectedFieldEditing : {}),
              }}
            >
              <div style={styles.selectedFieldHeader}>
                <div style={styles.selectedFieldInfo}>
                  <span style={styles.selectedFieldIcon}>
                    {field.calculation ? 'ƒ' : '📊'}
                  </span>
                  <span style={styles.selectedFieldName}>
                    {field.alias || field.field}
                  </span>
                  {field.aggregation && (
                    <span style={styles.aggregationBadge}>
                      {field.aggregation.toUpperCase()}
                    </span>
                  )}
                </div>
                {!readOnly && (
                  <div style={styles.selectedFieldActions}>
                    <button
                      onClick={() => moveField(index, 'up')}
                      disabled={index === 0}
                      style={styles.actionButton}
                      title="Move Up"
                    >
                      ↑
                    </button>
                    <button
                      onClick={() => moveField(index, 'down')}
                      disabled={index === selectedFields.length - 1}
                      style={styles.actionButton}
                      title="Move Down"
                    >
                      ↓
                    </button>
                    <button
                      onClick={() =>
                        setEditingField(editingField === field.field ? null : field.field)
                      }
                      style={styles.actionButton}
                      title="Edit"
                    >
                      ✎
                    </button>
                    <button
                      onClick={() => removeField(index)}
                      style={styles.deleteButton}
                      title="Remove"
                    >
                      ✕
                    </button>
                  </div>
                )}
              </div>

              {editingField === field.field && (
                <div style={styles.fieldEditor}>
                  <div style={styles.editorRow}>
                    <label style={styles.editorLabel}>Alias</label>
                    <input
                      type="text"
                      value={field.alias || ''}
                      onChange={(e) => updateField(index, { alias: e.target.value })}
                      style={styles.editorInput}
                      placeholder="Display name"
                    />
                  </div>

                  {showAggregations && !field.calculation && (
                    <div style={styles.editorRow}>
                      <label style={styles.editorLabel}>Aggregation</label>
                      <select
                        value={field.aggregation || ''}
                        onChange={(e) =>
                          updateField(index, {
                            aggregation: e.target.value as AggregationType || undefined,
                          })
                        }
                        style={styles.editorSelect}
                      >
                        <option value="">None</option>
                        <option value="sum">Sum</option>
                        <option value="avg">Average</option>
                        <option value="count">Count</option>
                        <option value="min">Minimum</option>
                        <option value="max">Maximum</option>
                        <option value="distinct">Distinct Count</option>
                        <option value="median">Median</option>
                      </select>
                    </div>
                  )}

                  <div style={styles.editorRow}>
                    <label style={styles.editorLabel}>Format</label>
                    <input
                      type="text"
                      value={field.format || ''}
                      onChange={(e) => updateField(index, { format: e.target.value })}
                      style={styles.editorInput}
                      placeholder="e.g., #,##0.00 or $#,##0"
                    />
                  </div>

                  {field.calculation && (
                    <div style={styles.editorRow}>
                      <label style={styles.editorLabel}>Expression</label>
                      <input
                        type="text"
                        value={field.calculation.expression}
                        readOnly
                        style={{...styles.editorInput, backgroundColor: '#f8fafc'}}
                      />
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}

          {selectedFields.length === 0 && (
            <div style={styles.emptyState}>
              <div style={styles.emptyStateIcon}>📊</div>
              <div style={styles.emptyStateText}>
                No fields selected
              </div>
              <div style={styles.emptyStateHint}>
                Click fields from the left panel to add them
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Calculation Builder Modal */}
      {showCalculationBuilder && (
        <CalculationBuilder
          availableTables={availableTables}
          selectedFields={selectedFields}
          onSave={addCalculatedField}
          onCancel={() => setShowCalculationBuilder(false)}
        />
      )}
    </div>
  );
};

// Calculation Builder Component
interface CalculationBuilderProps {
  availableTables: Table[];
  selectedFields: SelectField[];
  onSave: (calculation: { expression: string; fields: string[]; alias: string }) => void;
  onCancel: () => void;
}

const CalculationBuilder: React.FC<CalculationBuilderProps> = ({
  availableTables,
  selectedFields,
  onSave,
  onCancel,
}) => {
  const [alias, setAlias] = useState('');
  const [expression, setExpression] = useState('');
  const [selectedFieldsForCalc, setSelectedFieldsForCalc] = useState<string[]>([]);

  const handleSave = () => {
    if (!alias || !expression) {
      alert('Please provide both alias and expression');
      return;
    }

    onSave({ expression, fields: selectedFieldsForCalc, alias });
  };

  const insertFunction = (func: string) => {
    setExpression((prev) => prev + func + '()');
  };

  const insertField = (fieldPath: string) => {
    setExpression((prev) => prev + `{${fieldPath}}`);
    if (!selectedFieldsForCalc.includes(fieldPath)) {
      setSelectedFieldsForCalc((prev) => [...prev, fieldPath]);
    }
  };

  return (
    <div style={styles.modalOverlay}>
      <div style={styles.modalContent}>
        <div style={styles.modalHeader}>
          <h3 style={styles.modalTitle}>Calculated Field Builder</h3>
          <button onClick={onCancel} style={styles.modalCloseButton}>
            ✕
          </button>
        </div>

        <div style={styles.modalBody}>
          <div style={styles.formGroup}>
            <label style={styles.label}>Field Name</label>
            <input
              type="text"
              value={alias}
              onChange={(e) => setAlias(e.target.value)}
              style={styles.input}
              placeholder="e.g., Total Revenue"
            />
          </div>

          <div style={styles.formGroup}>
            <label style={styles.label}>Expression</label>
            <textarea
              value={expression}
              onChange={(e) => setExpression(e.target.value)}
              style={styles.textarea}
              placeholder="e.g., {sales.quantity} * {sales.price}"
              rows={4}
            />
          </div>

          <div style={styles.functionsPanel}>
            <div style={styles.functionsPanelTitle}>Functions</div>
            <div style={styles.functionsGrid}>
              {['SUM', 'AVG', 'MIN', 'MAX', 'COUNT', 'ROUND', 'ABS', 'SQRT'].map((func) => (
                <button
                  key={func}
                  onClick={() => insertFunction(func)}
                  style={styles.functionButton}
                >
                  {func}
                </button>
              ))}
            </div>
          </div>

          <div style={styles.fieldsPanel}>
            <div style={styles.fieldsPanelTitle}>Available Fields</div>
            <div style={styles.availableFieldsList}>
              {selectedFields.map((field, index) => (
                <button
                  key={index}
                  onClick={() => insertField(field.field)}
                  style={styles.fieldButton}
                >
                  {field.alias || field.field}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div style={styles.modalFooter}>
          <button onClick={onCancel} style={styles.cancelButton}>
            Cancel
          </button>
          <button onClick={handleSave} style={styles.saveButton}>
            Add Field
          </button>
        </div>
      </div>
    </div>
  );
};

// Helper functions
function getFieldTypeIcon(dataType: DataType): string {
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
    height: '100%',
    gap: '16px',
    fontFamily: 'Inter, system-ui, sans-serif',
  },
  availablePanel: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    backgroundColor: '#ffffff',
    border: '1px solid #e2e8f0',
    borderRadius: '8px',
    overflow: 'hidden',
  },
  selectedPanel: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    backgroundColor: '#ffffff',
    border: '1px solid #e2e8f0',
    borderRadius: '8px',
    overflow: 'hidden',
  },
  panelHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '12px 16px',
    borderBottom: '1px solid #e2e8f0',
    backgroundColor: '#f8fafc',
  },
  panelTitle: {
    fontSize: '14px',
    fontWeight: 600,
    margin: 0,
    color: '#1e293b',
  },
  searchInput: {
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '12px',
    width: '200px',
  },
  clearButton: {
    padding: '4px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    backgroundColor: '#ffffff',
    cursor: 'pointer',
    fontSize: '12px',
    color: '#ef4444',
  },
  fieldsList: {
    flex: 1,
    overflow: 'auto',
    padding: '8px',
  },
  tableGroup: {
    marginBottom: '8px',
  },
  tableGroupHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '8px',
    cursor: 'pointer',
    borderRadius: '4px',
    transition: 'background-color 0.2s',
  },
  expandIcon: {
    fontSize: '10px',
    color: '#64748b',
  },
  tableGroupName: {
    flex: 1,
    fontSize: '13px',
    fontWeight: 600,
    color: '#1e293b',
  },
  fieldCount: {
    fontSize: '11px',
    color: '#94a3b8',
  },
  tableFields: {
    paddingLeft: '16px',
  },
  fieldItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '6px 8px',
    borderRadius: '4px',
    marginBottom: '2px',
    transition: 'all 0.2s',
  },
  fieldItemSelected: {
    backgroundColor: '#eff6ff',
    borderLeft: '2px solid #2563eb',
  },
  fieldIcon: {
    fontSize: '14px',
  },
  fieldName: {
    flex: 1,
    fontSize: '12px',
    color: '#475569',
  },
  fieldType: {
    fontSize: '10px',
    color: '#94a3b8',
    textTransform: 'uppercase',
  },
  selectedBadge: {
    color: '#10b981',
    fontSize: '12px',
  },
  panelFooter: {
    padding: '12px 16px',
    borderTop: '1px solid #e2e8f0',
  },
  addCalculationButton: {
    width: '100%',
    padding: '8px',
    border: '1px dashed #2563eb',
    borderRadius: '6px',
    backgroundColor: '#eff6ff',
    color: '#2563eb',
    cursor: 'pointer',
    fontSize: '13px',
    fontWeight: 500,
  },
  selectedList: {
    flex: 1,
    overflow: 'auto',
    padding: '8px',
  },
  selectedField: {
    marginBottom: '8px',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    backgroundColor: '#ffffff',
    transition: 'all 0.2s',
  },
  selectedFieldEditing: {
    borderColor: '#2563eb',
    boxShadow: '0 0 0 3px rgba(37, 99, 235, 0.1)',
  },
  selectedFieldHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '10px 12px',
  },
  selectedFieldInfo: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    flex: 1,
  },
  selectedFieldIcon: {
    fontSize: '16px',
  },
  selectedFieldName: {
    fontSize: '13px',
    fontWeight: 500,
    color: '#1e293b',
  },
  aggregationBadge: {
    fontSize: '10px',
    padding: '2px 6px',
    backgroundColor: '#dbeafe',
    color: '#1e40af',
    borderRadius: '4px',
    fontWeight: 600,
  },
  selectedFieldActions: {
    display: 'flex',
    gap: '4px',
  },
  actionButton: {
    padding: '4px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    backgroundColor: '#ffffff',
    cursor: 'pointer',
    fontSize: '12px',
  },
  deleteButton: {
    padding: '4px 8px',
    border: '1px solid #fee2e2',
    borderRadius: '4px',
    backgroundColor: '#fef2f2',
    color: '#ef4444',
    cursor: 'pointer',
    fontSize: '12px',
  },
  fieldEditor: {
    padding: '12px',
    borderTop: '1px solid #e2e8f0',
    backgroundColor: '#f8fafc',
  },
  editorRow: {
    marginBottom: '12px',
  },
  editorLabel: {
    display: 'block',
    fontSize: '11px',
    fontWeight: 600,
    color: '#475569',
    marginBottom: '4px',
  },
  editorInput: {
    width: '100%',
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '12px',
  },
  editorSelect: {
    width: '100%',
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '12px',
    cursor: 'pointer',
  },
  emptyState: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '48px 16px',
    gap: '8px',
  },
  emptyStateIcon: {
    fontSize: '32px',
  },
  emptyStateText: {
    fontSize: '13px',
    color: '#64748b',
    fontWeight: 500,
  },
  emptyStateHint: {
    fontSize: '11px',
    color: '#94a3b8',
  },
  modalOverlay: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
  },
  modalContent: {
    backgroundColor: '#ffffff',
    borderRadius: '8px',
    width: '600px',
    maxHeight: '80vh',
    display: 'flex',
    flexDirection: 'column',
    boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1)',
  },
  modalHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '16px',
    borderBottom: '1px solid #e2e8f0',
  },
  modalTitle: {
    fontSize: '16px',
    fontWeight: 600,
    margin: 0,
    color: '#1e293b',
  },
  modalCloseButton: {
    border: 'none',
    background: 'none',
    fontSize: '20px',
    color: '#64748b',
    cursor: 'pointer',
  },
  modalBody: {
    flex: 1,
    overflow: 'auto',
    padding: '16px',
  },
  formGroup: {
    marginBottom: '16px',
  },
  label: {
    display: 'block',
    fontSize: '12px',
    fontWeight: 600,
    color: '#475569',
    marginBottom: '6px',
  },
  input: {
    width: '100%',
    padding: '8px 12px',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    fontSize: '14px',
  },
  textarea: {
    width: '100%',
    padding: '8px 12px',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    fontSize: '14px',
    fontFamily: 'monospace',
    resize: 'vertical',
  },
  functionsPanel: {
    marginBottom: '16px',
  },
  functionsPanelTitle: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#475569',
    marginBottom: '8px',
  },
  functionsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    gap: '4px',
  },
  functionButton: {
    padding: '6px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    backgroundColor: '#ffffff',
    cursor: 'pointer',
    fontSize: '11px',
    fontWeight: 500,
    color: '#2563eb',
  },
  fieldsPanel: {
    marginBottom: '16px',
  },
  fieldsPanelTitle: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#475569',
    marginBottom: '8px',
  },
  availableFieldsList: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: '4px',
  },
  fieldButton: {
    padding: '4px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    backgroundColor: '#f8fafc',
    cursor: 'pointer',
    fontSize: '11px',
  },
  modalFooter: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '8px',
    padding: '16px',
    borderTop: '1px solid #e2e8f0',
  },
  cancelButton: {
    padding: '8px 16px',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    backgroundColor: '#ffffff',
    cursor: 'pointer',
    fontSize: '14px',
  },
  saveButton: {
    padding: '8px 16px',
    border: 'none',
    borderRadius: '6px',
    backgroundColor: '#2563eb',
    color: '#ffffff',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 500,
  },
};

export default ReportFields;
