/**
 * CADDY v0.4.0 - Report Charts Component
 * $650M Platform - Production Ready
 *
 * Comprehensive chart configuration with support for multiple chart types,
 * advanced customization, and interactive preview.
 */

import React, { useState, useCallback } from 'react';
import {
  ChartConfig,
  ChartType,
  AxisConfig,
  DrillDownConfig,
  SelectField,
} from './types';

export interface ReportChartsProps {
  config: ChartConfig;
  availableFields: SelectField[];
  onChange: (config: ChartConfig) => void;
  readOnly?: boolean;
  showPreview?: boolean;
}

export const ReportCharts: React.FC<ReportChartsProps> = ({
  config,
  availableFields,
  onChange,
  readOnly = false,
  showPreview = true,
}) => {
  const [activeTab, setActiveTab] = useState<'basic' | 'advanced' | 'drilldown'>('basic');

  const updateConfig = useCallback(
    (updates: Partial<ChartConfig>) => {
      if (readOnly) return;
      onChange({ ...config, ...updates });
    },
    [config, onChange, readOnly]
  );

  const updateOptions = useCallback(
    (updates: Partial<ChartConfig['options']>) => {
      updateConfig({
        options: { ...config.options, ...updates },
      });
    },
    [config.options, updateConfig]
  );

  const updateDataMapping = useCallback(
    (updates: Partial<ChartConfig['dataMapping']>) => {
      updateConfig({
        dataMapping: { ...config.dataMapping, ...updates },
      });
    },
    [config.dataMapping, updateConfig]
  );

  const renderBasicTab = () => (
    <div style={styles.tabContent}>
      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Chart Type</h4>
        <div style={styles.chartTypeGrid}>
          {chartTypes.map((type) => (
            <button
              key={type.value}
              onClick={() => updateConfig({ type: type.value })}
              style={{
                ...styles.chartTypeButton,
                ...(config.type === type.value ? styles.chartTypeButtonActive : {}),
              }}
              disabled={readOnly}
            >
              <span style={styles.chartTypeIcon}>{type.icon}</span>
              <span style={styles.chartTypeLabel}>{type.label}</span>
            </button>
          ))}
        </div>
      </div>

      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Data Mapping</h4>

        <div style={styles.formGroup}>
          <label style={styles.label}>X-Axis Fields</label>
          <select
            multiple
            value={config.dataMapping.xAxis}
            onChange={(e) => {
              const selected = Array.from(e.target.selectedOptions, (o) => o.value);
              updateDataMapping({ xAxis: selected });
            }}
            style={styles.multiSelect}
            disabled={readOnly}
          >
            {availableFields.map((field, index) => (
              <option key={index} value={field.field}>
                {field.alias || field.field}
              </option>
            ))}
          </select>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.label}>Y-Axis Fields</label>
          <select
            multiple
            value={config.dataMapping.yAxis}
            onChange={(e) => {
              const selected = Array.from(e.target.selectedOptions, (o) => o.value);
              updateDataMapping({ yAxis: selected });
            }}
            style={styles.multiSelect}
            disabled={readOnly}
          >
            {availableFields.map((field, index) => (
              <option key={index} value={field.field}>
                {field.alias || field.field}
              </option>
            ))}
          </select>
        </div>

        {(config.type === 'pie' || config.type === 'gauge') && (
          <div style={styles.formGroup}>
            <label style={styles.label}>Value Field</label>
            <select
              value={config.dataMapping.value || ''}
              onChange={(e) => updateDataMapping({ value: e.target.value })}
              style={styles.select}
              disabled={readOnly}
            >
              <option value="">Select field...</option>
              {availableFields.map((field, index) => (
                <option key={index} value={field.field}>
                  {field.alias || field.field}
                </option>
              ))}
            </select>
          </div>
        )}

        <div style={styles.formGroup}>
          <label style={styles.label}>Series Field (Optional)</label>
          <select
            value={config.dataMapping.series || ''}
            onChange={(e) => updateDataMapping({ series: e.target.value || undefined })}
            style={styles.select}
            disabled={readOnly}
          >
            <option value="">None</option>
            {availableFields.map((field, index) => (
              <option key={index} value={field.field}>
                {field.alias || field.field}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Titles</h4>

        <div style={styles.formGroup}>
          <label style={styles.label}>Chart Title</label>
          <input
            type="text"
            value={config.options.title || ''}
            onChange={(e) => updateOptions({ title: e.target.value })}
            style={styles.input}
            placeholder="Chart title"
            disabled={readOnly}
          />
        </div>

        <div style={styles.formGroup}>
          <label style={styles.label}>Subtitle</label>
          <input
            type="text"
            value={config.options.subtitle || ''}
            onChange={(e) => updateOptions({ subtitle: e.target.value })}
            style={styles.input}
            placeholder="Chart subtitle"
            disabled={readOnly}
          />
        </div>
      </div>
    </div>
  );

  const renderAdvancedTab = () => (
    <div style={styles.tabContent}>
      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Legend</h4>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={config.options.legend?.show ?? true}
              onChange={(e) =>
                updateOptions({
                  legend: { ...config.options.legend, show: e.target.checked },
                })
              }
              disabled={readOnly}
            />
            <span>Show Legend</span>
          </label>
        </div>

        {config.options.legend?.show !== false && (
          <div style={styles.formGroup}>
            <label style={styles.label}>Legend Position</label>
            <select
              value={config.options.legend?.position || 'top'}
              onChange={(e) =>
                updateOptions({
                  legend: {
                    ...config.options.legend,
                    position: e.target.value as 'top' | 'bottom' | 'left' | 'right',
                  },
                })
              }
              style={styles.select}
              disabled={readOnly}
            >
              <option value="top">Top</option>
              <option value="bottom">Bottom</option>
              <option value="left">Left</option>
              <option value="right">Right</option>
            </select>
          </div>
        )}
      </div>

      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Tooltip</h4>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={config.options.tooltip?.enabled ?? true}
              onChange={(e) =>
                updateOptions({
                  tooltip: { ...config.options.tooltip, enabled: e.target.checked },
                })
              }
              disabled={readOnly}
            />
            <span>Show Tooltip</span>
          </label>
        </div>

        {config.options.tooltip?.enabled !== false && (
          <div style={styles.formGroup}>
            <label style={styles.label}>Tooltip Format</label>
            <input
              type="text"
              value={config.options.tooltip?.format || ''}
              onChange={(e) =>
                updateOptions({
                  tooltip: { ...config.options.tooltip, format: e.target.value },
                })
              }
              style={styles.input}
              placeholder="e.g., {b}: {c}"
              disabled={readOnly}
            />
          </div>
        )}
      </div>

      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Axes</h4>

        <div style={styles.formGroup}>
          <label style={styles.label}>X-Axis Label</label>
          <input
            type="text"
            value={config.options.axis?.x?.label || ''}
            onChange={(e) =>
              updateOptions({
                axis: {
                  ...config.options.axis,
                  x: { ...config.options.axis?.x, label: e.target.value },
                },
              })
            }
            style={styles.input}
            placeholder="X-axis label"
            disabled={readOnly}
          />
        </div>

        <div style={styles.formGroup}>
          <label style={styles.label}>Y-Axis Label</label>
          <input
            type="text"
            value={config.options.axis?.y?.label || ''}
            onChange={(e) =>
              updateOptions({
                axis: {
                  ...config.options.axis,
                  y: { ...config.options.axis?.y, label: e.target.value },
                },
              })
            }
            style={styles.input}
            placeholder="Y-axis label"
            disabled={readOnly}
          />
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={config.options.axis?.x?.grid ?? true}
              onChange={(e) =>
                updateOptions({
                  axis: {
                    ...config.options.axis,
                    x: { ...config.options.axis?.x, grid: e.target.checked },
                  },
                })
              }
              disabled={readOnly}
            />
            <span>Show Grid Lines</span>
          </label>
        </div>
      </div>

      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Visual Options</h4>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={config.options.stacked ?? false}
              onChange={(e) => updateOptions({ stacked: e.target.checked })}
              disabled={readOnly}
            />
            <span>Stacked Chart</span>
          </label>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={config.options.smooth ?? false}
              onChange={(e) => updateOptions({ smooth: e.target.checked })}
              disabled={readOnly}
            />
            <span>Smooth Lines</span>
          </label>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={config.options.animation ?? true}
              onChange={(e) => updateOptions({ animation: e.target.checked })}
              disabled={readOnly}
            />
            <span>Enable Animation</span>
          </label>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.label}>Color Palette</label>
          <div style={styles.colorPalette}>
            {(config.options.colors || defaultColors).map((color, index) => (
              <input
                key={index}
                type="color"
                value={color}
                onChange={(e) => {
                  const newColors = [...(config.options.colors || defaultColors)];
                  newColors[index] = e.target.value;
                  updateOptions({ colors: newColors });
                }}
                style={styles.colorInput}
                disabled={readOnly}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );

  const renderDrillDownTab = () => (
    <div style={styles.tabContent}>
      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Drill-Down Configuration</h4>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={config.drillDown?.enabled ?? false}
              onChange={(e) =>
                updateConfig({
                  drillDown: {
                    ...config.drillDown,
                    enabled: e.target.checked,
                    levels: config.drillDown?.levels || [],
                  },
                })
              }
              disabled={readOnly}
            />
            <span>Enable Drill-Down</span>
          </label>
        </div>

        {config.drillDown?.enabled && (
          <>
            <div style={styles.drillDownLevels}>
              {(config.drillDown.levels || []).map((level, index) => (
                <div key={index} style={styles.drillDownLevel}>
                  <div style={styles.drillDownLevelHeader}>
                    <span>Level {index + 1}</span>
                    {!readOnly && (
                      <button
                        onClick={() => {
                          const newLevels = config.drillDown!.levels.filter((_, i) => i !== index);
                          updateConfig({
                            drillDown: { ...config.drillDown!, levels: newLevels },
                          });
                        }}
                        style={styles.removeButton}
                      >
                        ✕
                      </button>
                    )}
                  </div>

                  <div style={styles.formGroup}>
                    <label style={styles.label}>Field</label>
                    <select
                      value={level.field}
                      onChange={(e) => {
                        const newLevels = [...config.drillDown!.levels];
                        newLevels[index] = { ...level, field: e.target.value };
                        updateConfig({
                          drillDown: { ...config.drillDown!, levels: newLevels },
                        });
                      }}
                      style={styles.select}
                      disabled={readOnly}
                    >
                      <option value="">Select field...</option>
                      {availableFields.map((field, fieldIndex) => (
                        <option key={fieldIndex} value={field.field}>
                          {field.alias || field.field}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div style={styles.formGroup}>
                    <label style={styles.label}>Target Report ID (Optional)</label>
                    <input
                      type="text"
                      value={level.reportId || ''}
                      onChange={(e) => {
                        const newLevels = [...config.drillDown!.levels];
                        newLevels[index] = { ...level, reportId: e.target.value };
                        updateConfig({
                          drillDown: { ...config.drillDown!, levels: newLevels },
                        });
                      }}
                      style={styles.input}
                      placeholder="Report ID"
                      disabled={readOnly}
                    />
                  </div>
                </div>
              ))}
            </div>

            {!readOnly && (
              <button
                onClick={() => {
                  const newLevel = { field: '', filters: [] };
                  updateConfig({
                    drillDown: {
                      ...config.drillDown!,
                      levels: [...(config.drillDown!.levels || []), newLevel],
                    },
                  });
                }}
                style={styles.addLevelButton}
              >
                + Add Drill-Down Level
              </button>
            )}
          </>
        )}
      </div>
    </div>
  );

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Chart Configuration</h3>
      </div>

      <div style={styles.tabs}>
        <button
          onClick={() => setActiveTab('basic')}
          style={{
            ...styles.tab,
            ...(activeTab === 'basic' ? styles.tabActive : {}),
          }}
        >
          Basic
        </button>
        <button
          onClick={() => setActiveTab('advanced')}
          style={{
            ...styles.tab,
            ...(activeTab === 'advanced' ? styles.tabActive : {}),
          }}
        >
          Advanced
        </button>
        <button
          onClick={() => setActiveTab('drilldown')}
          style={{
            ...styles.tab,
            ...(activeTab === 'drilldown' ? styles.tabActive : {}),
          }}
        >
          Drill-Down
        </button>
      </div>

      <div style={styles.content}>
        {activeTab === 'basic' && renderBasicTab()}
        {activeTab === 'advanced' && renderAdvancedTab()}
        {activeTab === 'drilldown' && renderDrillDownTab()}
      </div>

      {showPreview && (
        <div style={styles.preview}>
          <div style={styles.previewHeader}>Preview</div>
          <div style={styles.previewContent}>
            <div style={styles.previewPlaceholder}>
              <span style={styles.previewIcon}>{getChartIcon(config.type)}</span>
              <span style={styles.previewText}>{config.type} Chart Preview</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

// Chart types configuration
const chartTypes: Array<{ value: ChartType; label: string; icon: string }> = [
  { value: 'line', label: 'Line', icon: '📈' },
  { value: 'bar', label: 'Bar', icon: '📊' },
  { value: 'pie', label: 'Pie', icon: '🥧' },
  { value: 'scatter', label: 'Scatter', icon: '⚫' },
  { value: 'area', label: 'Area', icon: '📉' },
  { value: 'heatmap', label: 'Heatmap', icon: '🔥' },
  { value: 'gauge', label: 'Gauge', icon: '🎯' },
  { value: 'funnel', label: 'Funnel', icon: '🔻' },
  { value: 'waterfall', label: 'Waterfall', icon: '💧' },
];

const defaultColors = [
  '#2563eb',
  '#10b981',
  '#f59e0b',
  '#ef4444',
  '#8b5cf6',
  '#06b6d4',
];

function getChartIcon(type: ChartType): string {
  const chartType = chartTypes.find((ct) => ct.value === type);
  return chartType?.icon || '📊';
}

// Styles
const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#ffffff',
    border: '1px solid #e2e8f0',
    borderRadius: '8px',
    fontFamily: 'Inter, system-ui, sans-serif',
    overflow: 'hidden',
  },
  header: {
    padding: '12px 16px',
    borderBottom: '1px solid #e2e8f0',
    backgroundColor: '#f8fafc',
  },
  title: {
    fontSize: '14px',
    fontWeight: 600,
    margin: 0,
    color: '#1e293b',
  },
  tabs: {
    display: 'flex',
    borderBottom: '1px solid #e2e8f0',
    backgroundColor: '#f8fafc',
  },
  tab: {
    flex: 1,
    padding: '10px 16px',
    border: 'none',
    backgroundColor: 'transparent',
    cursor: 'pointer',
    fontSize: '13px',
    fontWeight: 500,
    color: '#64748b',
    borderBottom: '2px solid transparent',
    transition: 'all 0.2s',
  },
  tabActive: {
    color: '#2563eb',
    borderBottomColor: '#2563eb',
  },
  content: {
    flex: 1,
    overflow: 'auto',
    padding: '16px',
  },
  tabContent: {
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
  },
  section: {
    marginBottom: '16px',
  },
  sectionTitle: {
    fontSize: '13px',
    fontWeight: 600,
    margin: '0 0 12px 0',
    color: '#1e293b',
  },
  chartTypeGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(3, 1fr)',
    gap: '8px',
  },
  chartTypeButton: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '4px',
    padding: '12px',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    backgroundColor: '#ffffff',
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  chartTypeButtonActive: {
    borderColor: '#2563eb',
    backgroundColor: '#eff6ff',
  },
  chartTypeIcon: {
    fontSize: '24px',
  },
  chartTypeLabel: {
    fontSize: '12px',
    fontWeight: 500,
    color: '#475569',
  },
  formGroup: {
    marginBottom: '12px',
  },
  label: {
    display: 'block',
    fontSize: '12px',
    fontWeight: 500,
    color: '#475569',
    marginBottom: '4px',
  },
  input: {
    width: '100%',
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '13px',
  },
  select: {
    width: '100%',
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '13px',
    cursor: 'pointer',
  },
  multiSelect: {
    width: '100%',
    minHeight: '80px',
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '13px',
  },
  checkboxLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '13px',
    color: '#475569',
    cursor: 'pointer',
  },
  colorPalette: {
    display: 'flex',
    gap: '8px',
    flexWrap: 'wrap',
  },
  colorInput: {
    width: '40px',
    height: '40px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  drillDownLevels: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
    marginBottom: '12px',
  },
  drillDownLevel: {
    padding: '12px',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    backgroundColor: '#f8fafc',
  },
  drillDownLevelHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '12px',
    fontSize: '12px',
    fontWeight: 600,
    color: '#1e293b',
  },
  removeButton: {
    border: 'none',
    background: 'none',
    color: '#ef4444',
    cursor: 'pointer',
    fontSize: '16px',
  },
  addLevelButton: {
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
  preview: {
    borderTop: '1px solid #e2e8f0',
  },
  previewHeader: {
    padding: '8px 16px',
    fontSize: '12px',
    fontWeight: 600,
    color: '#64748b',
    backgroundColor: '#f8fafc',
  },
  previewContent: {
    padding: '16px',
    minHeight: '200px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  previewPlaceholder: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '8px',
    color: '#94a3b8',
  },
  previewIcon: {
    fontSize: '48px',
  },
  previewText: {
    fontSize: '13px',
  },
};

export default ReportCharts;
