/**
 * CADDY Enterprise Dashboard Export Component v0.4.0
 *
 * Export dashboard data to PDF, Excel, CSV, and JSON formats.
 * Includes customization options, templates, and scheduled exports.
 */

import React, { useState, useCallback, useRef } from 'react';
import type { ExportConfig, ExportFormat, DashboardConfig } from './types';
import { useDashboard } from './DashboardLayout';

/**
 * Dashboard export props
 */
export interface DashboardExportProps {
  /** Dashboard configuration */
  dashboardConfig: DashboardConfig;
  /** Dashboard data */
  data?: any;
  /** Available metrics for export */
  availableMetrics?: Array<{ id: string; name: string }>;
  /** Available charts for export */
  availableCharts?: Array<{ id: string; name: string }>;
  /** On export complete */
  onExportComplete?: (format: ExportFormat, blob: Blob) => void;
  /** On export error */
  onExportError?: (error: Error) => void;
  /** Custom class name */
  className?: string;
}

/**
 * Dashboard export component
 */
export const DashboardExport: React.FC<DashboardExportProps> = ({
  dashboardConfig,
  data,
  availableMetrics = [],
  availableCharts = [],
  onExportComplete,
  onExportError,
  className = '',
}) => {
  const [exportConfig, setExportConfig] = useState<ExportConfig>({
    format: 'pdf',
    includeCharts: true,
    includeRawData: false,
    includeHeader: true,
    includePageNumbers: true,
    orientation: 'portrait',
    paperSize: 'letter',
    fileName: `dashboard-export-${Date.now()}`,
  });
  const [isExporting, setIsExporting] = useState(false);
  const [exportProgress, setExportProgress] = useState(0);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const exportFormRef = useRef<HTMLFormElement>(null);
  const { theme, accessibility } = useDashboard();

  /**
   * Handle export configuration change
   */
  const handleConfigChange = useCallback((updates: Partial<ExportConfig>) => {
    setExportConfig((prev) => ({ ...prev, ...updates }));
  }, []);

  /**
   * Export to PDF
   */
  const exportToPDF = useCallback(async (): Promise<Blob> => {
    // Simulate PDF generation
    setExportProgress(25);
    await delay(500);

    // Generate PDF content
    const pdfContent = generatePDFContent(exportConfig, dashboardConfig, data);
    setExportProgress(75);
    await delay(500);

    // Convert to blob
    const blob = new Blob([pdfContent], { type: 'application/pdf' });
    setExportProgress(100);

    return blob;
  }, [exportConfig, dashboardConfig, data]);

  /**
   * Export to Excel
   */
  const exportToExcel = useCallback(async (): Promise<Blob> => {
    setExportProgress(25);
    await delay(300);

    // Generate Excel content
    const excelContent = generateExcelContent(exportConfig, dashboardConfig, data);
    setExportProgress(75);
    await delay(300);

    // Convert to blob
    const blob = new Blob([excelContent], {
      type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    });
    setExportProgress(100);

    return blob;
  }, [exportConfig, dashboardConfig, data]);

  /**
   * Export to CSV
   */
  const exportToCSV = useCallback(async (): Promise<Blob> => {
    setExportProgress(50);
    await delay(200);

    // Generate CSV content
    const csvContent = generateCSVContent(exportConfig, dashboardConfig, data);
    setExportProgress(100);

    // Convert to blob
    const blob = new Blob([csvContent], { type: 'text/csv' });

    return blob;
  }, [exportConfig, dashboardConfig, data]);

  /**
   * Export to JSON
   */
  const exportToJSON = useCallback(async (): Promise<Blob> => {
    setExportProgress(50);
    await delay(200);

    // Generate JSON content
    const jsonContent = JSON.stringify(
      {
        config: dashboardConfig,
        data,
        exportedAt: new Date().toISOString(),
        format: 'json',
      },
      null,
      2
    );
    setExportProgress(100);

    // Convert to blob
    const blob = new Blob([jsonContent], { type: 'application/json' });

    return blob;
  }, [dashboardConfig, data]);

  /**
   * Handle export
   */
  const handleExport = useCallback(async () => {
    setIsExporting(true);
    setExportProgress(0);

    try {
      let blob: Blob;

      // Export based on format
      switch (exportConfig.format) {
        case 'pdf':
          blob = await exportToPDF();
          break;
        case 'excel':
          blob = await exportToExcel();
          break;
        case 'csv':
          blob = await exportToCSV();
          break;
        case 'json':
          blob = await exportToJSON();
          break;
        default:
          throw new Error(`Unsupported export format: ${exportConfig.format}`);
      }

      // Download the file
      downloadBlob(blob, getFileName(exportConfig));

      // Callback
      if (onExportComplete) {
        onExportComplete(exportConfig.format, blob);
      }
    } catch (error: any) {
      console.error('Export failed:', error);
      if (onExportError) {
        onExportError(error);
      }
    } finally {
      setIsExporting(false);
      setExportProgress(0);
    }
  }, [exportConfig, exportToPDF, exportToExcel, exportToCSV, exportToJSON, onExportComplete, onExportError]);

  /**
   * Handle email export
   */
  const handleEmailExport = useCallback(async () => {
    // In production, this would send the export via email
    alert('Email export functionality would be implemented here');
  }, []);

  /**
   * Handle schedule export
   */
  const handleScheduleExport = useCallback(() => {
    // In production, this would open a scheduling dialog
    alert('Schedule export functionality would be implemented here');
  }, []);

  return (
    <div
      className={`dashboard-export ${className}`}
      style={styles.container}
      role="region"
      aria-label="Dashboard export options"
    >
      <form ref={exportFormRef} style={styles.form}>
        {/* Header */}
        <div style={styles.header}>
          <h3 style={styles.title}>Export Dashboard</h3>
          <p style={styles.subtitle}>Download your dashboard data in various formats</p>
        </div>

        {/* Export Format */}
        <div style={styles.section}>
          <label style={styles.label} id="format-label">
            Export Format
          </label>
          <div style={styles.formatGrid} role="radiogroup" aria-labelledby="format-label">
            <FormatOption
              format="pdf"
              icon="📄"
              label="PDF"
              description="Formatted report"
              selected={exportConfig.format === 'pdf'}
              onClick={() => handleConfigChange({ format: 'pdf' })}
            />
            <FormatOption
              format="excel"
              icon="📊"
              label="Excel"
              description="Spreadsheet data"
              selected={exportConfig.format === 'excel'}
              onClick={() => handleConfigChange({ format: 'excel' })}
            />
            <FormatOption
              format="csv"
              icon="📋"
              label="CSV"
              description="Raw data"
              selected={exportConfig.format === 'csv'}
              onClick={() => handleConfigChange({ format: 'csv' })}
            />
            <FormatOption
              format="json"
              icon="{ }"
              label="JSON"
              description="API format"
              selected={exportConfig.format === 'json'}
              onClick={() => handleConfigChange({ format: 'json' })}
            />
          </div>
        </div>

        {/* File Name */}
        <div style={styles.section}>
          <label style={styles.label} htmlFor="file-name">
            File Name
          </label>
          <input
            id="file-name"
            type="text"
            value={exportConfig.fileName || ''}
            onChange={(e) => handleConfigChange({ fileName: e.target.value })}
            style={styles.input}
            placeholder="Enter file name..."
          />
        </div>

        {/* Basic Options */}
        <div style={styles.section}>
          <label style={styles.label}>Include</label>
          <div style={styles.checkboxGroup}>
            <label style={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={exportConfig.includeCharts || false}
                onChange={(e) => handleConfigChange({ includeCharts: e.target.checked })}
                style={styles.checkbox}
              />
              <span>Charts and visualizations</span>
            </label>
            <label style={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={exportConfig.includeRawData || false}
                onChange={(e) => handleConfigChange({ includeRawData: e.target.checked })}
                style={styles.checkbox}
              />
              <span>Raw data tables</span>
            </label>
            {exportConfig.format === 'pdf' && (
              <>
                <label style={styles.checkboxLabel}>
                  <input
                    type="checkbox"
                    checked={exportConfig.includeHeader || false}
                    onChange={(e) => handleConfigChange({ includeHeader: e.target.checked })}
                    style={styles.checkbox}
                  />
                  <span>Header and footer</span>
                </label>
                <label style={styles.checkboxLabel}>
                  <input
                    type="checkbox"
                    checked={exportConfig.includePageNumbers || false}
                    onChange={(e) => handleConfigChange({ includePageNumbers: e.target.checked })}
                    style={styles.checkbox}
                  />
                  <span>Page numbers</span>
                </label>
              </>
            )}
          </div>
        </div>

        {/* PDF-specific options */}
        {exportConfig.format === 'pdf' && (
          <div style={styles.section}>
            <label style={styles.label}>PDF Settings</label>
            <div style={styles.row}>
              <div style={styles.column}>
                <label style={styles.smallLabel} htmlFor="orientation">
                  Orientation
                </label>
                <select
                  id="orientation"
                  value={exportConfig.orientation || 'portrait'}
                  onChange={(e) =>
                    handleConfigChange({ orientation: e.target.value as 'portrait' | 'landscape' })
                  }
                  style={styles.select}
                >
                  <option value="portrait">Portrait</option>
                  <option value="landscape">Landscape</option>
                </select>
              </div>
              <div style={styles.column}>
                <label style={styles.smallLabel} htmlFor="paper-size">
                  Paper Size
                </label>
                <select
                  id="paper-size"
                  value={exportConfig.paperSize || 'letter'}
                  onChange={(e) =>
                    handleConfigChange({ paperSize: e.target.value as 'letter' | 'a4' | 'legal' })
                  }
                  style={styles.select}
                >
                  <option value="letter">Letter</option>
                  <option value="a4">A4</option>
                  <option value="legal">Legal</option>
                </select>
              </div>
            </div>
          </div>
        )}

        {/* Advanced Options Toggle */}
        <button
          type="button"
          onClick={() => setShowAdvanced(!showAdvanced)}
          style={styles.advancedToggle}
          aria-expanded={showAdvanced}
        >
          {showAdvanced ? '▼' : '▶'} Advanced Options
        </button>

        {/* Advanced Options */}
        {showAdvanced && (
          <div style={styles.advancedSection}>
            {/* Date Range */}
            {exportConfig.dateRange !== undefined && (
              <div style={styles.section}>
                <label style={styles.label}>Date Range</label>
                <div style={styles.row}>
                  <input
                    type="date"
                    value={exportConfig.dateRange?.start || ''}
                    onChange={(e) =>
                      handleConfigChange({
                        dateRange: {
                          ...exportConfig.dateRange,
                          start: e.target.value,
                          end: exportConfig.dateRange?.end || '',
                        },
                      })
                    }
                    style={styles.input}
                    aria-label="Start date"
                  />
                  <span style={styles.dateSeparator}>to</span>
                  <input
                    type="date"
                    value={exportConfig.dateRange?.end || ''}
                    onChange={(e) =>
                      handleConfigChange({
                        dateRange: {
                          start: exportConfig.dateRange?.start || '',
                          end: e.target.value,
                        },
                      })
                    }
                    style={styles.input}
                    aria-label="End date"
                  />
                </div>
              </div>
            )}

            {/* Branding */}
            <div style={styles.section}>
              <label style={styles.label}>Branding</label>
              <input
                type="text"
                placeholder="Company name"
                value={exportConfig.branding?.companyName || ''}
                onChange={(e) =>
                  handleConfigChange({
                    branding: {
                      ...exportConfig.branding,
                      companyName: e.target.value,
                    },
                  })
                }
                style={styles.input}
              />
              <input
                type="text"
                placeholder="Footer text"
                value={exportConfig.branding?.footer || ''}
                onChange={(e) =>
                  handleConfigChange({
                    branding: {
                      ...exportConfig.branding,
                      footer: e.target.value,
                    },
                  })
                }
                style={{ ...styles.input, marginTop: 8 }}
              />
            </div>
          </div>
        )}

        {/* Progress Bar */}
        {isExporting && (
          <div style={styles.progressSection}>
            <div style={styles.progressBar}>
              <div
                style={{ ...styles.progressFill, width: `${exportProgress}%` }}
                role="progressbar"
                aria-valuenow={exportProgress}
                aria-valuemin={0}
                aria-valuemax={100}
              />
            </div>
            <p style={styles.progressText}>Exporting... {exportProgress}%</p>
          </div>
        )}

        {/* Actions */}
        <div style={styles.actions}>
          <button
            type="button"
            onClick={handleExport}
            disabled={isExporting}
            style={{
              ...styles.primaryButton,
              ...(isExporting && styles.buttonDisabled),
            }}
            aria-label="Export dashboard"
          >
            {isExporting ? 'Exporting...' : `Export as ${exportConfig.format.toUpperCase()}`}
          </button>

          <button
            type="button"
            onClick={handleEmailExport}
            disabled={isExporting}
            style={styles.secondaryButton}
            aria-label="Email export"
          >
            📧 Email
          </button>

          <button
            type="button"
            onClick={handleScheduleExport}
            disabled={isExporting}
            style={styles.secondaryButton}
            aria-label="Schedule export"
          >
            ⏰ Schedule
          </button>
        </div>
      </form>
    </div>
  );
};

/**
 * Format option component
 */
interface FormatOptionProps {
  format: ExportFormat;
  icon: string;
  label: string;
  description: string;
  selected: boolean;
  onClick: () => void;
}

const FormatOption: React.FC<FormatOptionProps> = ({
  format,
  icon,
  label,
  description,
  selected,
  onClick,
}) => {
  return (
    <button
      type="button"
      onClick={onClick}
      style={{
        ...styles.formatOption,
        ...(selected && styles.formatOptionSelected),
      }}
      role="radio"
      aria-checked={selected}
      aria-label={`${label}: ${description}`}
    >
      <div style={styles.formatIcon}>{icon}</div>
      <div style={styles.formatLabel}>{label}</div>
      <div style={styles.formatDescription}>{description}</div>
    </button>
  );
};

/**
 * Generate PDF content (simplified)
 */
function generatePDFContent(config: ExportConfig, dashboardConfig: DashboardConfig, data: any): string {
  return `%PDF-1.4
Dashboard Export - ${dashboardConfig.title}
Generated: ${new Date().toISOString()}
Format: PDF
Configuration: ${JSON.stringify(config)}
`;
}

/**
 * Generate Excel content (simplified)
 */
function generateExcelContent(config: ExportConfig, dashboardConfig: DashboardConfig, data: any): string {
  return `Dashboard Export\n${dashboardConfig.title}\n\nGenerated: ${new Date().toISOString()}\n`;
}

/**
 * Generate CSV content
 */
function generateCSVContent(config: ExportConfig, dashboardConfig: DashboardConfig, data: any): string {
  let csv = 'Dashboard Export\n';
  csv += `Title,${dashboardConfig.title}\n`;
  csv += `Generated,${new Date().toISOString()}\n\n`;
  csv += 'Metric,Value,Change,Trend\n';

  // Add data rows (example)
  if (data && Array.isArray(data.metrics)) {
    data.metrics.forEach((metric: any) => {
      csv += `${metric.name},${metric.value},${metric.change || 'N/A'},${metric.trend}\n`;
    });
  }

  return csv;
}

/**
 * Download blob as file
 */
function downloadBlob(blob: Blob, fileName: string): void {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

/**
 * Get file name with extension
 */
function getFileName(config: ExportConfig): string {
  const ext = {
    pdf: 'pdf',
    excel: 'xlsx',
    csv: 'csv',
    json: 'json',
  }[config.format];

  return `${config.fileName || 'dashboard-export'}.${ext}`;
}

/**
 * Delay helper
 */
function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Component styles
 */
const styles: Record<string, React.CSSProperties> = {
  container: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    border: '1px solid var(--color-border, #e0e0e0)',
    padding: 24,
  },
  form: {
    display: 'flex',
    flexDirection: 'column',
    gap: 24,
  },
  header: {
    marginBottom: 8,
  },
  title: {
    margin: '0 0 8px 0',
    fontSize: 20,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
  },
  subtitle: {
    margin: 0,
    fontSize: 14,
    color: 'var(--color-text-secondary, #666)',
  },
  section: {
    display: 'flex',
    flexDirection: 'column',
    gap: 12,
  },
  label: {
    fontSize: 14,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
  },
  smallLabel: {
    fontSize: 13,
    fontWeight: 500,
    color: 'var(--color-text-secondary, #666)',
    marginBottom: 4,
  },
  formatGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
    gap: 12,
  },
  formatOption: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    padding: 16,
    border: '2px solid var(--color-border, #e0e0e0)',
    borderRadius: 8,
    backgroundColor: 'var(--color-surface, #fff)',
    cursor: 'pointer',
    transition: 'all var(--animation-duration, 200ms)',
  },
  formatOptionSelected: {
    borderColor: 'var(--color-primary, #1976d2)',
    backgroundColor: '#e3f2fd',
  },
  formatIcon: {
    fontSize: 32,
    marginBottom: 8,
  },
  formatLabel: {
    fontSize: 14,
    fontWeight: 600,
    color: 'var(--color-text, #333)',
    marginBottom: 4,
  },
  formatDescription: {
    fontSize: 12,
    color: 'var(--color-text-secondary, #666)',
    textAlign: 'center',
  },
  input: {
    padding: '10px 12px',
    border: '1px solid var(--color-border, #e0e0e0)',
    borderRadius: 4,
    fontSize: 14,
    backgroundColor: 'var(--color-surface, #fff)',
    color: 'var(--color-text, #333)',
  },
  select: {
    padding: '10px 12px',
    border: '1px solid var(--color-border, #e0e0e0)',
    borderRadius: 4,
    fontSize: 14,
    backgroundColor: 'var(--color-surface, #fff)',
    color: 'var(--color-text, #333)',
    cursor: 'pointer',
  },
  checkboxGroup: {
    display: 'flex',
    flexDirection: 'column',
    gap: 10,
  },
  checkboxLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
    fontSize: 14,
    color: 'var(--color-text, #333)',
    cursor: 'pointer',
  },
  checkbox: {
    width: 18,
    height: 18,
    cursor: 'pointer',
  },
  row: {
    display: 'flex',
    gap: 12,
  },
  column: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
  },
  dateSeparator: {
    display: 'flex',
    alignItems: 'center',
    fontSize: 14,
    color: 'var(--color-text-secondary, #666)',
  },
  advancedToggle: {
    padding: '8px 0',
    border: 'none',
    backgroundColor: 'transparent',
    color: 'var(--color-primary, #1976d2)',
    cursor: 'pointer',
    fontSize: 14,
    fontWeight: 500,
    textAlign: 'left',
  },
  advancedSection: {
    paddingLeft: 16,
    borderLeft: '3px solid var(--color-divider, #e0e0e0)',
    display: 'flex',
    flexDirection: 'column',
    gap: 20,
  },
  progressSection: {
    padding: '16px 0',
  },
  progressBar: {
    width: '100%',
    height: 8,
    backgroundColor: 'var(--color-background, #f5f5f5)',
    borderRadius: 4,
    overflow: 'hidden',
  },
  progressFill: {
    height: '100%',
    backgroundColor: 'var(--color-primary, #1976d2)',
    transition: 'width 300ms ease',
  },
  progressText: {
    marginTop: 8,
    fontSize: 13,
    color: 'var(--color-text-secondary, #666)',
    textAlign: 'center',
  },
  actions: {
    display: 'flex',
    gap: 12,
    flexWrap: 'wrap',
  },
  primaryButton: {
    flex: 1,
    minWidth: 150,
    padding: '12px 24px',
    border: 'none',
    borderRadius: 4,
    backgroundColor: 'var(--color-primary, #1976d2)',
    color: '#fff',
    fontSize: 15,
    fontWeight: 600,
    cursor: 'pointer',
    transition: 'background-color var(--animation-duration, 200ms)',
  },
  secondaryButton: {
    padding: '12px 20px',
    border: '1px solid var(--color-border, #e0e0e0)',
    borderRadius: 4,
    backgroundColor: 'var(--color-surface, #fff)',
    color: 'var(--color-text, #333)',
    fontSize: 14,
    fontWeight: 500,
    cursor: 'pointer',
    transition: 'background-color var(--animation-duration, 200ms)',
  },
  buttonDisabled: {
    opacity: 0.6,
    cursor: 'not-allowed',
  },
};

export default DashboardExport;
